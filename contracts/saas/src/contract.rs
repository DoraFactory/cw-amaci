#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, coins, from_json, to_json_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env,
    MessageInfo, Order, Reply, Response, StdError, StdResult, SubMsg, SubMsgResponse, Timestamp,
    Uint128, Uint256, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::{may_pay, parse_instantiate_response_data};

// External contract types with aliases to avoid path conflicts
use cw_amaci::state::RoundInfo;
use cw_oracle_maci::msg::{
    InstantiateMsg as OracleMaciInstantiateMsg, InstantiationData as OracleMaciInstantiationData,
    VotingPowerArgs,
};
use cw_oracle_maci::state::{
    PubKey as OracleMaciPubKey, RoundInfo as OracleMaciRoundInfo, VotingPowerMode,
    VotingTime as OracleMaciVotingTime,
};

// Local contract types
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, InstantiationData, MigrateMsg, PubKey, QueryMsg};
use crate::state::{
    Config, FeeGrantRecord, MaciContractInfo, OperatorInfo, CONFIG, FEEGRANT_RECORDS,
    MACI_CONTRACTS, MACI_CONTRACT_COUNTER, OPERATORS, TOTAL_BALANCE,
};

// Version info for migration
const CONTRACT_NAME: &str = "crates.io:cw-saas";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Reply IDs
pub const CREATED_ORACLE_MACI_ROUND_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        admin: msg.admin,
        registry_contract: msg.registry_contract,
        denom: msg.denom,
    };

    CONFIG.save(deps.storage, &config)?;
    TOTAL_BALANCE.save(deps.storage, &Uint128::zero())?;
    MACI_CONTRACT_COUNTER.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", config.admin.to_string())
        .add_attribute("denom", config.denom))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            admin,
            registry_contract,
            denom,
        } => execute_update_config(deps, info, admin, registry_contract, denom),
        ExecuteMsg::AddOperator { operator } => execute_add_operator(deps, env, info, operator),
        ExecuteMsg::RemoveOperator { operator } => {
            execute_remove_operator(deps, env, info, operator)
        }
        ExecuteMsg::Deposit {} => execute_deposit(deps, env, info),
        ExecuteMsg::Withdraw { amount, recipient } => {
            execute_withdraw(deps, env, info, amount, recipient)
        }
        ExecuteMsg::BatchFeegrant { recipients, amount } => {
            execute_batch_feegrant(deps, env, info, recipients, amount)
        }
        ExecuteMsg::BatchFeeGrantToOperators { amount } => {
            execute_batch_feegrant_to_operators(deps, env, info, amount)
        }
        ExecuteMsg::CreateOracleMaciRound {
            oracle_maci_code_id,
            coordinator,
            max_voters,
            vote_option_map,
            round_info,
            start_time,
            end_time,
            circuit_type,
            certification_system,
            whitelist_backend_pubkey,
        } => execute_create_oracle_maci_round(
            deps,
            env,
            info,
            oracle_maci_code_id,
            coordinator,
            max_voters,
            vote_option_map,
            round_info,
            start_time,
            end_time,
            circuit_type,
            certification_system,
            whitelist_backend_pubkey,
        ),
        ExecuteMsg::ExecuteContract {
            contract_addr,
            msg,
            funds,
        } => execute_execute_contract(deps, env, info, contract_addr, msg, funds),
        ExecuteMsg::SetOracleMaciRoundInfo {
            contract_addr,
            round_info,
        } => execute_set_oracle_maci_round_info(deps, env, info, contract_addr, round_info),
        ExecuteMsg::SetOracleMaciVoteOptionMap {
            contract_addr,
            vote_option_map,
        } => {
            execute_set_oracle_maci_vote_option_map(deps, env, info, contract_addr, vote_option_map)
        }
        ExecuteMsg::GrantOracleMaciFeegrant {
            contract_addr,
            grantee,
            base_amount,
        } => {
            execute_grant_oracle_maci_feegrant(deps, env, info, contract_addr, grantee, base_amount)
        }
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    admin: Option<Addr>,
    registry_contract: Option<Addr>,
    denom: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // Only admin can update config
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(admin) = admin {
        config.admin = admin;
    }
    if let Some(registry_contract) = registry_contract {
        config.registry_contract = Some(registry_contract);
    }
    if let Some(denom) = denom {
        config.denom = denom;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn execute_add_operator(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only admin can add operators
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Check if operator already exists
    if OPERATORS.has(deps.storage, &operator) {
        return Err(ContractError::OperatorAlreadyExists {});
    }

    let operator_info = OperatorInfo {
        address: operator.clone(),
        added_at: env.block.time,
        active: true,
    };

    OPERATORS.save(deps.storage, &operator, &operator_info)?;

    Ok(Response::new()
        .add_attribute("action", "add_operator")
        .add_attribute("operator", operator.to_string()))
}

pub fn execute_remove_operator(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only admin can remove operators
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Check if operator exists
    if !OPERATORS.has(deps.storage, &operator) {
        return Err(ContractError::OperatorNotFound {});
    }

    OPERATORS.remove(deps.storage, &operator);

    Ok(Response::new()
        .add_attribute("action", "remove_operator")
        .add_attribute("operator", operator.to_string()))
}

pub fn execute_deposit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Check if funds were sent
    let amount = may_pay(&info, &config.denom)?;
    if amount.is_zero() {
        return Err(ContractError::NoFunds {});
    }

    // Update total balance
    let mut total_balance = TOTAL_BALANCE.load(deps.storage)?;
    total_balance += amount;
    TOTAL_BALANCE.save(deps.storage, &total_balance)?;

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("amount", amount.to_string())
        .add_attribute("total_balance", total_balance.to_string()))
}

pub fn execute_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
    recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only admin can withdraw
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    if amount.is_zero() {
        return Err(ContractError::InvalidWithdrawAmount {});
    }

    // Check if sufficient balance
    let total_balance = TOTAL_BALANCE.load(deps.storage)?;
    if total_balance < amount {
        return Err(ContractError::InsufficientBalance {});
    }

    // Update total balance
    let new_balance = total_balance - amount;
    TOTAL_BALANCE.save(deps.storage, &new_balance)?;

    // Send funds to recipient
    let recipient_addr = recipient.unwrap_or_else(|| info.sender.clone());
    let msg = BankMsg::Send {
        to_address: recipient_addr.to_string(),
        amount: vec![Coin {
            denom: config.denom,
            amount,
        }],
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "withdraw")
        .add_attribute("amount", amount.to_string())
        .add_attribute("recipient", recipient_addr.to_string())
        .add_attribute("new_balance", new_balance.to_string()))
}

pub fn execute_batch_feegrant(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipients: Vec<Addr>,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only admin can grant fees
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    if amount.is_zero() {
        return Err(ContractError::InvalidFeegrantAmount {});
    }

    if recipients.is_empty() {
        return Err(ContractError::EmptyAddressList {});
    }

    // For now, we'll record the feegrant intention
    // In a real implementation, this would interact with the cosmos feegrant module
    for recipient in &recipients {
        let record = FeeGrantRecord {
            grantee: recipient.clone(),
            amount,
            granted_at: env.block.time,
            granted_by: info.sender.clone(),
        };
        FEEGRANT_RECORDS.save(deps.storage, recipient, &record)?;
    }

    Ok(Response::new()
        .add_attribute("action", "batch_feegrant")
        .add_attribute("recipients_count", recipients.len().to_string())
        .add_attribute("amount_per_recipient", amount.to_string())
        .add_attribute(
            "total_amount",
            amount
                .checked_mul(Uint128::from(recipients.len() as u128))?
                .to_string(),
        ))
}

pub fn execute_batch_feegrant_to_operators(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only admin can grant fees
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    if amount.is_zero() {
        return Err(ContractError::InvalidFeegrantAmount {});
    }

    // Get all active operators
    let operators: Vec<Addr> = OPERATORS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(addr, _)| addr))
        .collect::<StdResult<Vec<_>>>()?;

    if operators.is_empty() {
        return Err(ContractError::EmptyAddressList {});
    }

    // Record feegrant for each operator
    for operator in &operators {
        let record = FeeGrantRecord {
            grantee: operator.clone(),
            amount,
            granted_at: env.block.time,
            granted_by: info.sender.clone(),
        };
        FEEGRANT_RECORDS.save(deps.storage, operator, &record)?;
    }

    Ok(Response::new()
        .add_attribute("action", "batch_feegrant_to_operators")
        .add_attribute("operators_count", operators.len().to_string())
        .add_attribute("amount_per_operator", amount.to_string())
        .add_attribute(
            "total_amount",
            amount
                .checked_mul(Uint128::from(operators.len() as u128))?
                .to_string(),
        ))
}

pub fn execute_create_oracle_maci_round(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    oracle_maci_code_id: u64,
    coordinator: PubKey,
    max_voters: u128,
    vote_option_map: Vec<String>,
    round_info: RoundInfo,
    start_time: Timestamp,
    end_time: Timestamp,
    circuit_type: Uint256,
    certification_system: Uint256,
    whitelist_backend_pubkey: String,
) -> Result<Response, ContractError> {
    // Only operators can create Oracle MACI rounds
    if !OPERATORS.has(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Calculate deployment fee - base fee of 50 DORA for contract deployment
    let deployment_fee = Uint128::from(50000000000000000000u128); // 50 DORA

    // Calculate required token amount for Oracle MACI (max_voters * 10 DORA)
    let token_amount = Uint128::from(max_voters as u128 * 10000000000000000000u128); // max_voters * 10 DORA

    // Total required amount = deployment fee + token amount
    let total_required = deployment_fee + token_amount;

    // Check if SaaS has sufficient balance
    let total_balance = TOTAL_BALANCE.load(deps.storage)?;
    if total_balance < total_required {
        return Err(ContractError::InsufficientFundsForRound {
            required: total_required,
            available: total_balance,
        });
    }

    // Update total balance immediately (deduct the total cost)
    let new_balance = total_balance - total_required;
    TOTAL_BALANCE.save(deps.storage, &new_balance)?;

    // Create Oracle MACI VotingTime using provided start_time and end_time
    let oracle_voting_time = OracleMaciVotingTime {
        start_time: start_time,
        end_time: end_time,
    };

    // Create Oracle MACI InstantiateMsg using proper Oracle MACI types (like registry does with AMACI)
    let oracle_maci_instantiate_msg = OracleMaciInstantiateMsg {
        coordinator: OracleMaciPubKey {
            x: coordinator.x,
            y: coordinator.y,
        },
        max_voters,
        vote_option_map: vote_option_map.clone(),
        round_info: OracleMaciRoundInfo {
            title: round_info.title.clone(),
            description: round_info.description.clone(),
            link: round_info.link.clone(),
        },
        voting_time: oracle_voting_time,
        circuit_type,
        certification_system,
        whitelist_backend_pubkey: whitelist_backend_pubkey.clone(),
        // 写死的默认值 - 1人1票系统
        whitelist_ecosystem: "doravota".to_string(),
        whitelist_snapshot_height: Uint256::zero(),
        whitelist_voting_power_args: VotingPowerArgs {
            mode: VotingPowerMode::Slope,
            slope: Uint256::one(),
            threshold: Uint256::one(),
        },
        feegrant_operator: env.contract.address.clone(),
    };

    // Validate the message can be serialized properly
    let serialized_msg = to_json_binary(&oracle_maci_instantiate_msg).map_err(|e| {
        ContractError::SerializationError {
            msg: format!("Failed to serialize Oracle MACI InstantiateMsg: {}", e),
        }
    })?;

    // Prepare the instantiate message with SaaS contract as admin and token funds
    let instantiate_msg = WasmMsg::Instantiate {
        admin: Some(env.contract.address.to_string()), // SaaS合约作为Oracle MACI的admin
        code_id: oracle_maci_code_id,
        msg: serialized_msg,
        funds: coins(token_amount.u128(), "peaka"), // Send all fees, including admin_fee
        label: format!("Oracle Maci Round - {}", round_info.title),
    };

    // Get the next MACI contract counter
    let mut maci_counter = MACI_CONTRACT_COUNTER.load(deps.storage)?;
    maci_counter += 1;
    MACI_CONTRACT_COUNTER.save(deps.storage, &maci_counter)?;

    // Save MACI contract info with temporary address (will be updated in reply)
    let maci_contract_info = MaciContractInfo {
        contract_address: Addr::unchecked("pending"), // 临时地址，将在reply中更新
        creator_operator: info.sender.clone(),
        round_title: round_info.title.clone(),
        created_at: env.block.time,
        code_id: oracle_maci_code_id,
        creation_fee: total_required,
    };
    MACI_CONTRACTS.save(deps.storage, maci_counter, &maci_contract_info)?;

    // Create SubMsg with reply using registry pattern - 这样可以获取真实的合约地址
    let submsg = SubMsg::reply_on_success(instantiate_msg, CREATED_ORACLE_MACI_ROUND_REPLY_ID);
    Ok(Response::new()
        .add_submessage(submsg)
        .add_attribute("action", "create_oracle_maci_round")
        .add_attribute("operator", info.sender.to_string())
        .add_attribute("round_title", round_info.title)
        .add_attribute("deployment_fee", deployment_fee.to_string())
        .add_attribute("token_amount", token_amount.to_string())
        .add_attribute("total_cost", total_required.to_string())
        .add_attribute("new_balance", new_balance.to_string())
        .add_attribute("max_voters", max_voters.to_string())
        .add_attribute("maci_counter", maci_counter.to_string()))
}

pub fn execute_execute_contract(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract_addr: String,
    msg: Binary,
    funds: Vec<Coin>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only operators can execute contracts
    if !OPERATORS.has(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Validate the contract address format
    let target_addr = deps.api.addr_validate(&contract_addr)?;

    // Calculate total funds amount for recording
    let total_amount = funds
        .iter()
        .filter(|coin| coin.denom == config.denom)
        .map(|coin| coin.amount)
        .fold(Uint128::zero(), |acc, amount| acc + amount);

    // If funds are being sent with the message, check if SaaS has sufficient balance
    if !total_amount.is_zero() {
        let total_balance = TOTAL_BALANCE.load(deps.storage)?;
        if total_balance < total_amount {
            return Err(ContractError::InsufficientBalance {});
        }

        // Update total balance
        let new_balance = total_balance - total_amount;
        TOTAL_BALANCE.save(deps.storage, &new_balance)?;
    }

    // Execute the contract call
    let execute_msg = WasmMsg::Execute {
        contract_addr: target_addr.to_string(),
        msg,
        funds,
    };

    Ok(Response::new()
        .add_message(execute_msg)
        .add_attribute("action", "execute_contract")
        .add_attribute("operator", info.sender.to_string())
        .add_attribute("target_contract", contract_addr)
        .add_attribute("funds_amount", total_amount.to_string()))
}

pub fn execute_set_oracle_maci_round_info(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract_addr: String,
    round_info: RoundInfo,
) -> Result<Response, ContractError> {
    // Only operators can manage Oracle MACI contracts
    if !OPERATORS.has(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Validate the contract address format
    let target_addr = deps.api.addr_validate(&contract_addr)?;

    // Create Oracle MACI SetRoundInfo message
    let oracle_maci_msg = serde_json::json!({
        "set_round_info": {
            "round_info": {
                "title": round_info.title,
                "description": round_info.description,
                "link": round_info.link
            }
        }
    });

    // Execute the contract call
    let execute_msg = WasmMsg::Execute {
        contract_addr: target_addr.to_string(),
        msg: to_json_binary(&oracle_maci_msg)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(execute_msg)
        .add_attribute("action", "set_oracle_maci_round_info")
        .add_attribute("operator", info.sender.to_string())
        .add_attribute("target_contract", contract_addr)
        .add_attribute("round_title", round_info.title))
}

pub fn execute_set_oracle_maci_vote_option_map(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract_addr: String,
    vote_option_map: Vec<String>,
) -> Result<Response, ContractError> {
    // Only operators can manage Oracle MACI contracts
    if !OPERATORS.has(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Validate the contract address format
    let target_addr = deps.api.addr_validate(&contract_addr)?;

    // Create Oracle MACI SetVoteOptionsMap message
    let oracle_maci_msg = serde_json::json!({
        "set_vote_options_map": {
            "vote_option_map": vote_option_map
        }
    });

    // Execute the contract call
    let execute_msg = WasmMsg::Execute {
        contract_addr: target_addr.to_string(),
        msg: to_json_binary(&oracle_maci_msg)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(execute_msg)
        .add_attribute("action", "set_oracle_maci_vote_option_map")
        .add_attribute("operator", info.sender.to_string())
        .add_attribute("target_contract", contract_addr)
        .add_attribute("vote_options_count", vote_option_map.len().to_string()))
}

pub fn execute_grant_oracle_maci_feegrant(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract_addr: String,
    grantee: Addr,
    base_amount: Uint128,
) -> Result<Response, ContractError> {
    // Only operators can manage Oracle MACI feegrants
    if !OPERATORS.has(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Validate the contract address format
    let target_addr = deps.api.addr_validate(&contract_addr)?;

    // Create Oracle MACI Grant message
    let oracle_maci_msg = serde_json::json!({
        "grant": {
            "base_amount": base_amount.to_string(),
            "grantee": grantee.to_string()
        }
    });

    // Execute the contract call
    let execute_msg = WasmMsg::Execute {
        contract_addr: target_addr.to_string(),
        msg: to_json_binary(&oracle_maci_msg)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(execute_msg)
        .add_attribute("action", "grant_oracle_maci_feegrant")
        .add_attribute("operator", info.sender.to_string())
        .add_attribute("target_contract", contract_addr)
        .add_attribute("grantee", grantee.to_string())
        .add_attribute("base_amount", base_amount.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::Operators {} => to_json_binary(&query_operators(deps)?),
        QueryMsg::IsOperator { address } => to_json_binary(&query_is_operator(deps, address)?),
        QueryMsg::Balance {} => to_json_binary(&TOTAL_BALANCE.load(deps.storage)?),
        QueryMsg::FeeGrantRecords { start_after, limit } => {
            to_json_binary(&query_feegrant_records(deps, start_after, limit)?)
        }
        QueryMsg::MaciContracts { start_after, limit } => {
            to_json_binary(&query_maci_contracts(deps, start_after, limit)?)
        }
        QueryMsg::OperatorMaciContracts {
            operator,
            start_after,
            limit,
        } => to_json_binary(&query_operator_maci_contracts(
            deps,
            operator,
            start_after,
            limit,
        )?),
        QueryMsg::MaciContract { contract_id } => {
            to_json_binary(&query_maci_contract(deps, contract_id)?)
        }
    }
}

fn query_operators(deps: Deps) -> StdResult<Vec<OperatorInfo>> {
    OPERATORS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_, operator_info)| operator_info))
        .collect()
}

fn query_is_operator(deps: Deps, address: Addr) -> StdResult<bool> {
    Ok(OPERATORS.has(deps.storage, &address))
}

fn query_feegrant_records(
    deps: Deps,
    start_after: Option<Addr>,
    limit: Option<u32>,
) -> StdResult<Vec<FeeGrantRecord>> {
    let limit = limit.unwrap_or(30).min(100) as usize;
    let start = start_after.as_ref().map(Bound::exclusive);

    FEEGRANT_RECORDS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, record)| record))
        .collect()
}

fn query_maci_contracts(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<MaciContractInfo>> {
    let limit = limit.unwrap_or(30).min(100) as usize;
    let start = start_after.map(|s| s + 1);

    MACI_CONTRACTS
        .range(
            deps.storage,
            start.map(|s| Bound::exclusive(s)),
            None,
            Order::Ascending,
        )
        .take(limit)
        .map(|item| item.map(|(_, info)| info))
        .collect()
}

fn query_operator_maci_contracts(
    deps: Deps,
    operator: Addr,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<MaciContractInfo>> {
    let limit = limit.unwrap_or(30).min(100) as usize;
    let start = start_after.map(|s| s + 1);

    MACI_CONTRACTS
        .range(
            deps.storage,
            start.map(|s| Bound::exclusive(s)),
            None,
            Order::Ascending,
        )
        .filter(|item| {
            if let Ok((_, info)) = item {
                info.creator_operator == operator
            } else {
                false
            }
        })
        .take(limit)
        .map(|item| item.map(|(_, info)| info))
        .collect()
}

fn query_maci_contract(deps: Deps, contract_id: u64) -> StdResult<Option<MaciContractInfo>> {
    MACI_CONTRACTS.may_load(deps.storage, contract_id)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        CREATED_ORACLE_MACI_ROUND_REPLY_ID => {
            reply_created_oracle_maci_round(deps, env, msg.result.into_result())
        }
        id => Err(ContractError::Std(StdError::generic_err(format!(
            "Unknown reply id: {}",
            id
        )))),
    }
}

fn reply_created_oracle_maci_round(
    deps: DepsMut,
    _env: Env,
    result: Result<SubMsgResponse, String>,
) -> Result<Response, ContractError> {
    // 解析SubMsg响应
    let response = result.map_err(StdError::generic_err)?;

    // 使用和registry相同的方式解析响应数据
    let data = response
        .data
        .ok_or(ContractError::Std(StdError::generic_err(
            "Data missing from response",
        )))?;
    let parsed_response = match parse_instantiate_response_data(&data) {
        Ok(data) => data,
        Err(err) => {
            return Err(ContractError::Std(StdError::generic_err(format!(
                "Failed to parse instantiate response: {}",
                err
            ))))
        }
    };

    let contract_address = Addr::unchecked(parsed_response.contract_address.clone());

    let oracle_maci_return_data: OracleMaciInstantiationData =
        from_json(&parsed_response.data.unwrap())?;

    // 获取当前的MACI合约计数器
    let maci_counter = MACI_CONTRACT_COUNTER.load(deps.storage)?;

    // 更新MACI合约记录中的合约地址（从临时地址更新为真实地址）
    let mut maci_contract_info = MACI_CONTRACTS.load(deps.storage, maci_counter)?;
    maci_contract_info.contract_address = contract_address.clone();
    MACI_CONTRACTS.save(deps.storage, maci_counter, &maci_contract_info)?;

    // 准备返回数据 - 现在包含完整的oracle maci实例化数据
    let saas_instantiation_data = InstantiationData {
        addr: contract_address.clone(),
    };

    let mut response_attrs = vec![
        attr("action", "created_oracle_maci_round"),
        attr("round_addr", &contract_address.to_string()),
        attr("caller", &oracle_maci_return_data.caller.to_string()),
        attr("admin", &oracle_maci_return_data.caller.to_string()),
        attr("operator", &oracle_maci_return_data.caller.to_string()),
        attr("maci_counter", maci_counter.to_string()),
    ];

    // 如果成功解析了Oracle MACI的实例化数据，添加更多详细信息
    // if let Some(ref oracle_maci_data) = oracle_maci_instantiation_data {
    response_attrs.extend(vec![
        attr(
            "voting_start",
            &oracle_maci_return_data
                .voting_time
                .start_time
                .nanos()
                .to_string(),
        ),
        attr(
            "voting_end",
            &oracle_maci_return_data
                .voting_time
                .end_time
                .nanos()
                .to_string(),
        ),
        attr(
            "round_title",
            &oracle_maci_return_data.round_info.title.to_string(),
        ),
        attr("max_voters", oracle_maci_return_data.max_voters.to_string()),
        attr(
            "vote_option_map",
            format!("{:?}", oracle_maci_return_data.vote_option_map),
        ),
        attr("circuit_type", &oracle_maci_return_data.circuit_type),
        attr(
            "certification_system",
            &oracle_maci_return_data.certification_system,
        ),
        attr(
            "coordinator_pubkey_x",
            oracle_maci_return_data.coordinator.x.to_string(),
        ),
        attr(
            "coordinator_pubkey_y",
            oracle_maci_return_data.coordinator.y.to_string(),
        ),
        attr(
            "state_tree_depth",
            oracle_maci_return_data
                .parameters
                .state_tree_depth
                .to_string(),
        ),
        attr(
            "int_state_tree_depth",
            &oracle_maci_return_data
                .parameters
                .int_state_tree_depth
                .to_string(),
        ),
        attr(
            "vote_option_tree_depth",
            oracle_maci_return_data
                .parameters
                .vote_option_tree_depth
                .to_string(),
        ),
        attr(
            "message_batch_size",
            &oracle_maci_return_data
                .parameters
                .message_batch_size
                .to_string(),
        ),
    ]);

    if oracle_maci_return_data.round_info.description != "" {
        response_attrs.push(attr(
            "round_description",
            &oracle_maci_return_data.round_info.description,
        ));
    }

    if oracle_maci_return_data.round_info.link != "" {
        response_attrs.push(attr("round_link", &oracle_maci_return_data.round_info.link));
    }

    Ok(Response::new()
        .add_attributes(response_attrs)
        .set_data(to_json_binary(&saas_instantiation_data)?))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    cw2::ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "migrate"),
        attr("version", CONTRACT_VERSION),
    ]))
}

// Utility functions
pub fn validate_dora_address(address: &str) -> Result<(), ContractError> {
    match bech32::decode(address) {
        Ok((prefix, _data, _variant)) => {
            if prefix != "dora" {
                return Err(ContractError::InvalidAddressPrefix {
                    expected: "dora".to_string(),
                    actual: prefix,
                });
            }
            Ok(())
        }
        Err(_) => Err(ContractError::InvalidAddress {
            address: address.to_string(),
        }),
    }
}
