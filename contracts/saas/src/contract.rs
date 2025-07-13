#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn,
    Response, StdResult, SubMsg, SubMsgResult, Uint128, Uint256, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::may_pay;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{
    Config, ConsumptionRecord, FeeGrantRecord, MaciContractInfo, OperatorInfo, CONFIG,
    CONSUMPTION_COUNTER, CONSUMPTION_RECORDS, FEEGRANT_RECORDS, MACI_CONTRACTS,
    MACI_CONTRACTS_BY_OPERATOR, MACI_CONTRACT_COUNTER, OPERATORS, TOTAL_BALANCE,
};

// Version info for migration
const CONTRACT_NAME: &str = "crates.io:cw-saas";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    CONSUMPTION_COUNTER.save(deps.storage, &0u64)?;
    MACI_CONTRACT_COUNTER.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
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
        ExecuteMsg::CreateAmaciRound {
            max_voter,
            max_option,
            voice_credit_amount,
            round_info,
            voting_time,
            whitelist,
            pre_deactivate_root,
            circuit_type,
            certification_system,
        } => execute_create_amaci_round(
            deps,
            env,
            info,
            max_voter,
            max_option,
            voice_credit_amount,
            round_info,
            voting_time,
            whitelist,
            pre_deactivate_root,
            circuit_type,
            certification_system,
        ),
        ExecuteMsg::CreateMaciRound {
            maci_code_id,
            parameters,
            coordinator,
            qtr_lib,
            groth16_process_vkey,
            groth16_tally_vkey,
            plonk_process_vkey,
            plonk_tally_vkey,
            max_vote_options,
            round_info,
            voting_time,
            whitelist,
            circuit_type,
            certification_system,
            admin_override,
            label,
        } => execute_create_maci_round(
            deps,
            env,
            info,
            maci_code_id,
            parameters,
            coordinator,
            qtr_lib,
            groth16_process_vkey,
            groth16_tally_vkey,
            plonk_process_vkey,
            plonk_tally_vkey,
            max_vote_options,
            round_info,
            voting_time,
            whitelist,
            circuit_type,
            certification_system,
            admin_override,
            label,
        ),
        ExecuteMsg::CreateOracleMaciRound {
            oracle_maci_code_id,
            coordinator,
            max_voters,
            vote_option_map,
            round_info,
            voting_time,
            circuit_type,
            certification_system,
            whitelist_backend_pubkey,
            whitelist_ecosystem,
            whitelist_snapshot_height,
            whitelist_voting_power_args,
        } => execute_create_oracle_maci_round(
            deps,
            env,
            info,
            oracle_maci_code_id,
            coordinator,
            max_voters,
            vote_option_map,
            round_info,
            voting_time,
            circuit_type,
            certification_system,
            whitelist_backend_pubkey,
            whitelist_ecosystem,
            whitelist_snapshot_height,
            whitelist_voting_power_args,
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

    Ok(Response::new().add_attribute("method", "update_config"))
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

    // Record the add operator operation
    let mut counter = CONSUMPTION_COUNTER.load(deps.storage)?;
    counter += 1;
    CONSUMPTION_COUNTER.save(deps.storage, &counter)?;

    let record = ConsumptionRecord {
        operator: info.sender.clone(),
        action: "add_operator".to_string(),
        amount: Uint128::zero(), // No cost for operator management
        timestamp: env.block.time,
        description: format!("Add operator {}", operator),
    };

    CONSUMPTION_RECORDS.save(deps.storage, counter, &record)?;

    Ok(Response::new()
        .add_attribute("method", "add_operator")
        .add_attribute("operator", operator.to_string()))
}

pub fn execute_remove_operator(
    deps: DepsMut,
    env: Env,
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

    // Record the remove operator operation
    let mut counter = CONSUMPTION_COUNTER.load(deps.storage)?;
    counter += 1;
    CONSUMPTION_COUNTER.save(deps.storage, &counter)?;

    let record = ConsumptionRecord {
        operator: info.sender.clone(),
        action: "remove_operator".to_string(),
        amount: Uint128::zero(), // No cost for operator management
        timestamp: env.block.time,
        description: format!("Remove operator {}", operator),
    };

    CONSUMPTION_RECORDS.save(deps.storage, counter, &record)?;

    Ok(Response::new()
        .add_attribute("method", "remove_operator")
        .add_attribute("operator", operator.to_string()))
}

pub fn execute_deposit(
    deps: DepsMut,
    env: Env,
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

    // Record the deposit
    let mut counter = CONSUMPTION_COUNTER.load(deps.storage)?;
    counter += 1;
    CONSUMPTION_COUNTER.save(deps.storage, &counter)?;

    let record = ConsumptionRecord {
        operator: info.sender.clone(),
        action: "deposit".to_string(),
        amount,
        timestamp: env.block.time,
        description: format!("Deposit {} {}", amount, config.denom),
    };

    CONSUMPTION_RECORDS.save(deps.storage, counter, &record)?;

    Ok(Response::new()
        .add_attribute("method", "deposit")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("amount", amount.to_string())
        .add_attribute("total_balance", total_balance.to_string()))
}

pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
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

    // Record the withdrawal
    let mut counter = CONSUMPTION_COUNTER.load(deps.storage)?;
    counter += 1;
    CONSUMPTION_COUNTER.save(deps.storage, &counter)?;

    let recipient_addr = recipient.unwrap_or_else(|| info.sender.clone());
    let record = ConsumptionRecord {
        operator: info.sender.clone(),
        action: "withdraw".to_string(),
        amount,
        timestamp: env.block.time,
        description: format!("Withdraw {} {} to {}", amount, config.denom, recipient_addr),
    };

    CONSUMPTION_RECORDS.save(deps.storage, counter, &record)?;

    // Send funds to recipient
    let msg = BankMsg::Send {
        to_address: recipient_addr.to_string(),
        amount: vec![Coin {
            denom: config.denom,
            amount,
        }],
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("method", "withdraw")
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

    // Record the consumption
    let mut counter = CONSUMPTION_COUNTER.load(deps.storage)?;
    counter += 1;
    CONSUMPTION_COUNTER.save(deps.storage, &counter)?;

    let total_amount = amount.checked_mul(Uint128::from(recipients.len() as u128))?;
    let record = ConsumptionRecord {
        operator: info.sender.clone(),
        action: "batch_feegrant".to_string(),
        amount: total_amount,
        timestamp: env.block.time,
        description: format!(
            "Batch feegrant {} {} to {} recipients",
            amount,
            config.denom,
            recipients.len()
        ),
    };

    CONSUMPTION_RECORDS.save(deps.storage, counter, &record)?;

    Ok(Response::new()
        .add_attribute("method", "batch_feegrant")
        .add_attribute("recipients_count", recipients.len().to_string())
        .add_attribute("amount_per_recipient", amount.to_string())
        .add_attribute("total_amount", total_amount.to_string()))
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
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
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

    // Record the consumption
    let mut counter = CONSUMPTION_COUNTER.load(deps.storage)?;
    counter += 1;
    CONSUMPTION_COUNTER.save(deps.storage, &counter)?;

    let total_amount = amount.checked_mul(Uint128::from(operators.len() as u128))?;
    let record = ConsumptionRecord {
        operator: info.sender.clone(),
        action: "batch_feegrant_operators".to_string(),
        amount: total_amount,
        timestamp: env.block.time,
        description: format!(
            "Batch feegrant {} {} to {} operators",
            amount,
            config.denom,
            operators.len()
        ),
    };

    CONSUMPTION_RECORDS.save(deps.storage, counter, &record)?;

    Ok(Response::new()
        .add_attribute("method", "batch_feegrant_to_operators")
        .add_attribute("operators_count", operators.len().to_string())
        .add_attribute("amount_per_operator", amount.to_string())
        .add_attribute("total_amount", total_amount.to_string()))
}

pub fn execute_create_amaci_round(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    max_voter: Uint256,
    max_option: Uint256,
    voice_credit_amount: Uint256,
    round_info: cw_amaci::state::RoundInfo,
    voting_time: cw_amaci::state::VotingTime,
    whitelist: Option<cw_amaci::msg::WhitelistBase>,
    pre_deactivate_root: Uint256,
    circuit_type: Uint256,
    certification_system: Uint256,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only operators can create rounds
    if !OPERATORS.has(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Check if registry contract is set
    let registry_contract = config
        .registry_contract
        .ok_or(ContractError::NoRegistryContract {})?;

    // Calculate required fee based on circuit size (same logic as in registry contract)
    let required_fee = if max_voter <= Uint256::from_u128(25u128)
        && max_option <= Uint256::from_u128(5u128)
    {
        Uint128::from(20000000000000000000u128) // 20 DORA
    } else if max_voter <= Uint256::from_u128(625u128) && max_option <= Uint256::from_u128(25u128) {
        Uint128::from(750000000000000000000u128) // 750 DORA
    } else {
        return Err(ContractError::InsufficientBalance {});
    };

    // Check if SaaS has sufficient balance
    let total_balance = TOTAL_BALANCE.load(deps.storage)?;
    if total_balance < required_fee {
        return Err(ContractError::InsufficientFundsForRound {
            required: required_fee,
            available: total_balance,
        });
    }

    // Update total balance
    let new_balance = total_balance - required_fee;
    TOTAL_BALANCE.save(deps.storage, &new_balance)?;

    // Record the consumption
    let mut counter = CONSUMPTION_COUNTER.load(deps.storage)?;
    counter += 1;
    CONSUMPTION_COUNTER.save(deps.storage, &counter)?;

    let record = ConsumptionRecord {
        operator: info.sender.clone(),
        action: "create_amaci_round".to_string(),
        amount: required_fee,
        timestamp: env.block.time,
        description: format!(
            "Create AMACI round '{}' - fee: {} {}",
            round_info.title, required_fee, config.denom
        ),
    };

    CONSUMPTION_RECORDS.save(deps.storage, counter, &record)?;

    // Create registry ExecuteMsg using serde_json
    let registry_msg = serde_json::json!({
        "create_round": {
            "operator": info.sender,
            "max_voter": max_voter.to_string(),
            "max_option": max_option.to_string(),
            "voice_credit_amount": voice_credit_amount.to_string(),
            "round_info": {
                "title": round_info.title,
                "description": round_info.description,
                "link": round_info.link
            },
            "voting_time": {
                "start_time": voting_time.start_time.nanos().to_string(),
                "end_time": voting_time.end_time.nanos().to_string()
            },
            "whitelist": whitelist,
            "pre_deactivate_root": pre_deactivate_root.to_string(),
            "circuit_type": circuit_type.to_string(),
            "certification_system": certification_system.to_string()
        }
    });

    let submsg = SubMsg::new(WasmMsg::Execute {
        contract_addr: registry_contract.to_string(),
        msg: to_json_binary(&registry_msg)?,
        funds: vec![Coin {
            denom: config.denom,
            amount: required_fee,
        }],
    });

    Ok(Response::new()
        .add_submessage(submsg)
        .add_attribute("method", "create_amaci_round")
        .add_attribute("operator", info.sender.to_string())
        .add_attribute("required_fee", required_fee.to_string())
        .add_attribute("new_balance", new_balance.to_string()))
}

pub fn execute_execute_contract(
    deps: DepsMut,
    env: Env,
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

    // Record the execution
    let mut counter = CONSUMPTION_COUNTER.load(deps.storage)?;
    counter += 1;
    CONSUMPTION_COUNTER.save(deps.storage, &counter)?;

    let record = ConsumptionRecord {
        operator: info.sender.clone(),
        action: "execute_contract".to_string(),
        amount: total_amount,
        timestamp: env.block.time,
        description: format!(
            "Execute contract {} with {} {} funds",
            contract_addr, total_amount, config.denom
        ),
    };

    CONSUMPTION_RECORDS.save(deps.storage, counter, &record)?;

    // Execute the contract call
    let execute_msg = WasmMsg::Execute {
        contract_addr: target_addr.to_string(),
        msg,
        funds,
    };

    Ok(Response::new()
        .add_message(execute_msg)
        .add_attribute("method", "execute_contract")
        .add_attribute("operator", info.sender.to_string())
        .add_attribute("target_contract", contract_addr)
        .add_attribute("funds_amount", total_amount.to_string()))
}

pub fn execute_set_oracle_maci_round_info(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract_addr: String,
    round_info: cw_amaci::state::RoundInfo,
) -> Result<Response, ContractError> {
    // Only operators can manage Oracle MACI contracts
    if !OPERATORS.has(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Validate the contract address format
    let target_addr = deps.api.addr_validate(&contract_addr)?;

    // Record the management operation
    let mut counter = CONSUMPTION_COUNTER.load(deps.storage)?;
    counter += 1;
    CONSUMPTION_COUNTER.save(deps.storage, &counter)?;

    let record = ConsumptionRecord {
        operator: info.sender.clone(),
        action: "set_oracle_maci_round_info".to_string(),
        amount: Uint128::zero(), // No cost for management operations
        timestamp: env.block.time,
        description: format!(
            "Set Oracle MACI round info for contract {} - title: '{}'",
            contract_addr, round_info.title
        ),
    };

    CONSUMPTION_RECORDS.save(deps.storage, counter, &record)?;

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
        .add_attribute("method", "set_oracle_maci_round_info")
        .add_attribute("operator", info.sender.to_string())
        .add_attribute("target_contract", contract_addr)
        .add_attribute("round_title", round_info.title))
}

pub fn execute_set_oracle_maci_vote_option_map(
    deps: DepsMut,
    env: Env,
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

    // Record the management operation
    let mut counter = CONSUMPTION_COUNTER.load(deps.storage)?;
    counter += 1;
    CONSUMPTION_COUNTER.save(deps.storage, &counter)?;

    let record = ConsumptionRecord {
        operator: info.sender.clone(),
        action: "set_oracle_maci_vote_option_map".to_string(),
        amount: Uint128::zero(), // No cost for management operations
        timestamp: env.block.time,
        description: format!(
            "Set Oracle MACI vote option map for contract {} - {} options",
            contract_addr,
            vote_option_map.len()
        ),
    };

    CONSUMPTION_RECORDS.save(deps.storage, counter, &record)?;

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
        .add_attribute("method", "set_oracle_maci_vote_option_map")
        .add_attribute("operator", info.sender.to_string())
        .add_attribute("target_contract", contract_addr)
        .add_attribute("vote_options_count", vote_option_map.len().to_string()))
}

pub fn execute_grant_oracle_maci_feegrant(
    deps: DepsMut,
    env: Env,
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

    // Record the management operation
    let mut counter = CONSUMPTION_COUNTER.load(deps.storage)?;
    counter += 1;
    CONSUMPTION_COUNTER.save(deps.storage, &counter)?;

    let record = ConsumptionRecord {
        operator: info.sender.clone(),
        action: "grant_oracle_maci_feegrant".to_string(),
        amount: Uint128::zero(), // No cost for management operations
        timestamp: env.block.time,
        description: format!(
            "Grant Oracle MACI feegrant for contract {} - grantee: {}, amount: {}",
            contract_addr, grantee, base_amount
        ),
    };

    CONSUMPTION_RECORDS.save(deps.storage, counter, &record)?;

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
        .add_attribute("method", "grant_oracle_maci_feegrant")
        .add_attribute("operator", info.sender.to_string())
        .add_attribute("target_contract", contract_addr)
        .add_attribute("grantee", grantee.to_string())
        .add_attribute("base_amount", base_amount.to_string()))
}

pub fn execute_create_maci_round(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    maci_code_id: u64,
    parameters: crate::msg::MaciParameters,
    coordinator: crate::msg::PubKey,
    qtr_lib: crate::msg::QuinaryTreeRoot,
    groth16_process_vkey: Option<crate::msg::Groth16VKeyType>,
    groth16_tally_vkey: Option<crate::msg::Groth16VKeyType>,
    plonk_process_vkey: Option<crate::msg::PlonkVKeyType>,
    plonk_tally_vkey: Option<crate::msg::PlonkVKeyType>,
    max_vote_options: Uint256,
    round_info: cw_amaci::state::RoundInfo,
    voting_time: Option<crate::msg::MaciVotingTime>,
    whitelist: Option<crate::msg::Whitelist>,
    circuit_type: Uint256,
    certification_system: Uint256,
    admin_override: Option<Addr>,
    label: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only operators can create MACI rounds
    if !OPERATORS.has(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Calculate deployment fee - base fee of 50 DORA for contract deployment
    let deployment_fee = Uint128::from(50000000000000000000u128); // 50 DORA

    // Check if SaaS has sufficient balance
    let total_balance = TOTAL_BALANCE.load(deps.storage)?;
    if total_balance < deployment_fee {
        return Err(ContractError::InsufficientFundsForRound {
            required: deployment_fee,
            available: total_balance,
        });
    }

    // Update total balance
    let new_balance = total_balance - deployment_fee;
    TOTAL_BALANCE.save(deps.storage, &new_balance)?;

    // Record the consumption
    let mut counter = CONSUMPTION_COUNTER.load(deps.storage)?;
    counter += 1;
    CONSUMPTION_COUNTER.save(deps.storage, &counter)?;

    let record = ConsumptionRecord {
        operator: info.sender.clone(),
        action: "create_maci_round".to_string(),
        amount: deployment_fee,
        timestamp: env.block.time,
        description: format!(
            "Create MACI round '{}' - deployment fee: {} {}",
            round_info.title, deployment_fee, config.denom
        ),
    };

    CONSUMPTION_RECORDS.save(deps.storage, counter, &record)?;

    // Convert VotingTime if provided
    let maci_voting_time = voting_time.map(|vt| {
        serde_json::json!({
            "start_time": vt.start_time.map(|t| t.nanos().to_string()),
            "end_time": vt.end_time.map(|t| t.nanos().to_string())
        })
    });

    // Convert Whitelist if provided
    let maci_whitelist = whitelist.map(|wl| {
        serde_json::json!({
            "users": wl.users.into_iter().map(|user| {
                serde_json::json!({
                    "addr": user.addr,
                    "balance": user.balance.to_string()
                })
            }).collect::<Vec<_>>()
        })
    });

    // Create MACI InstantiateMsg
    let maci_instantiate_msg = serde_json::json!({
        "parameters": {
            "state_tree_depth": parameters.state_tree_depth.to_string(),
            "int_state_tree_depth": parameters.int_state_tree_depth.to_string(),
            "message_batch_size": parameters.message_batch_size.to_string(),
            "vote_option_tree_depth": parameters.vote_option_tree_depth.to_string()
        },
        "coordinator": {
            "x": coordinator.x.to_string(),
            "y": coordinator.y.to_string()
        },
        "qtr_lib": {
            "zeros": qtr_lib.zeros.iter().map(|z| z.to_string()).collect::<Vec<_>>()
        },
        "groth16_process_vkey": groth16_process_vkey,
        "groth16_tally_vkey": groth16_tally_vkey,
        "plonk_process_vkey": plonk_process_vkey,
        "plonk_tally_vkey": plonk_tally_vkey,
        "max_vote_options": max_vote_options.to_string(),
        "round_info": {
            "title": round_info.title,
            "description": round_info.description,
            "link": round_info.link
        },
        "voting_time": maci_voting_time,
        "whitelist": maci_whitelist,
        "circuit_type": circuit_type.to_string(),
        "certification_system": certification_system.to_string()
    });

    // Get the next MACI contract counter
    let mut maci_counter = MACI_CONTRACT_COUNTER.load(deps.storage)?;
    maci_counter += 1;
    MACI_CONTRACT_COUNTER.save(deps.storage, &maci_counter)?;

    // Prepare the instantiate message
    let instantiate_msg = WasmMsg::Instantiate {
        admin: admin_override.map(|a| a.to_string()),
        code_id: maci_code_id,
        msg: to_json_binary(&maci_instantiate_msg)?,
        funds: vec![],
        label: format!("{}_{}", label, maci_counter),
    };

    // Create SubMsg with reply to capture the new contract address
    let submsg = SubMsg {
        id: maci_counter,
        msg: instantiate_msg.into(),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new()
        .add_submessage(submsg)
        .add_attribute("method", "create_maci_round")
        .add_attribute("operator", info.sender.to_string())
        .add_attribute("round_title", round_info.title)
        .add_attribute("deployment_fee", deployment_fee.to_string())
        .add_attribute("new_balance", new_balance.to_string())
        .add_attribute("maci_counter", maci_counter.to_string()))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_create_oracle_maci_round(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    oracle_maci_code_id: u64,
    coordinator: crate::msg::PubKey,
    max_voters: u128,
    vote_option_map: Vec<String>,
    round_info: cw_amaci::state::RoundInfo,
    voting_time: Option<crate::msg::MaciVotingTime>,
    circuit_type: Uint256,
    certification_system: Uint256,
    whitelist_backend_pubkey: String,
    whitelist_ecosystem: String,
    whitelist_snapshot_height: Uint256,
    whitelist_voting_power_args: crate::msg::VotingPowerArgs,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

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

    // Update total balance
    let new_balance = total_balance - total_required;
    TOTAL_BALANCE.save(deps.storage, &new_balance)?;

    // Record the consumption
    let mut counter = CONSUMPTION_COUNTER.load(deps.storage)?;
    counter += 1;
    CONSUMPTION_COUNTER.save(deps.storage, &counter)?;

    let record = ConsumptionRecord {
        operator: info.sender.clone(),
        action: "create_oracle_maci_round".to_string(),
        amount: total_required,
        timestamp: env.block.time,
        description: format!(
            "Create Oracle MACI round '{}' - deployment fee: {} {}, token amount: {} {} (total: {} {})",
            round_info.title, deployment_fee, config.denom, token_amount, config.denom, total_required, config.denom
        ),
    };

    CONSUMPTION_RECORDS.save(deps.storage, counter, &record)?;

    // Convert voting time if provided
    let oracle_voting_time = voting_time.map(|vt| {
        serde_json::json!({
            "start_time": vt.start_time.map(|t| t.nanos().to_string()),
            "end_time": vt.end_time.map(|t| t.nanos().to_string())
        })
    });

    // Create Oracle MACI InstantiateMsg
    let oracle_maci_instantiate_msg = serde_json::json!({
        "coordinator": {
            "x": coordinator.x.to_string(),
            "y": coordinator.y.to_string()
        },
        "max_voters": max_voters,
        "vote_option_map": vote_option_map,
        "round_info": {
            "title": round_info.title,
            "description": round_info.description,
            "link": round_info.link
        },
        "voting_time": oracle_voting_time,
        "circuit_type": circuit_type.to_string(),
        "certification_system": certification_system.to_string(),
        "whitelist_backend_pubkey": whitelist_backend_pubkey,
        "whitelist_ecosystem": whitelist_ecosystem,
        "whitelist_snapshot_height": whitelist_snapshot_height.to_string(),
        "whitelist_voting_power_args": {
            "mode": whitelist_voting_power_args.mode,
            "slope": whitelist_voting_power_args.slope.to_string(),
            "threshold": whitelist_voting_power_args.threshold.to_string()
        },
        "feegrant_operator": env.contract.address.to_string() // SaaS合约作为feegrant operator
    });

    // Get the next MACI contract counter
    let mut maci_counter = MACI_CONTRACT_COUNTER.load(deps.storage)?;
    maci_counter += 1;
    MACI_CONTRACT_COUNTER.save(deps.storage, &maci_counter)?;

    // Prepare the instantiate message with SaaS contract as admin and token funds
    let instantiate_msg = WasmMsg::Instantiate {
        admin: Some(env.contract.address.to_string()), // SaaS合约作为Oracle MACI的admin
        code_id: oracle_maci_code_id,
        msg: to_json_binary(&oracle_maci_instantiate_msg)?,
        funds: vec![Coin {
            denom: config.denom.clone(),
            amount: token_amount,
        }],
        label: format!("Oracle Maci Round - {}", round_info.title),
    };

    // Create SubMsg with reply to capture the new contract address
    let submsg = SubMsg {
        id: maci_counter,
        msg: instantiate_msg.into(),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new()
        .add_submessage(submsg)
        .add_attribute("method", "create_oracle_maci_round")
        .add_attribute("operator", info.sender.to_string())
        .add_attribute("round_title", round_info.title)
        .add_attribute("deployment_fee", deployment_fee.to_string())
        .add_attribute("token_amount", token_amount.to_string())
        .add_attribute("total_cost", total_required.to_string())
        .add_attribute("max_voters", max_voters.to_string())
        .add_attribute("new_balance", new_balance.to_string())
        .add_attribute("maci_counter", maci_counter.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::Operators {} => to_json_binary(&query_operators(deps)?),
        QueryMsg::IsOperator { address } => to_json_binary(&query_is_operator(deps, address)?),
        QueryMsg::Balance {} => to_json_binary(&TOTAL_BALANCE.load(deps.storage)?),
        QueryMsg::ConsumptionRecords { start_after, limit } => {
            to_json_binary(&query_consumption_records(deps, start_after, limit)?)
        }
        QueryMsg::FeeGrantRecords { start_after, limit } => {
            to_json_binary(&query_feegrant_records(deps, start_after, limit)?)
        }
        QueryMsg::OperatorConsumptionRecords {
            operator,
            start_after,
            limit,
        } => to_json_binary(&query_operator_consumption_records(
            deps,
            operator,
            start_after,
            limit,
        )?),
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
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| item.map(|(_, operator_info)| operator_info))
        .collect()
}

fn query_is_operator(deps: Deps, address: Addr) -> StdResult<bool> {
    Ok(OPERATORS.has(deps.storage, &address))
}

fn query_consumption_records(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<ConsumptionRecord>> {
    let limit = limit.unwrap_or(30).min(100) as usize;
    let start = start_after.map(|s| s + 1);

    CONSUMPTION_RECORDS
        .range(
            deps.storage,
            start.map(|s| Bound::exclusive(s)),
            None,
            cosmwasm_std::Order::Ascending,
        )
        .take(limit)
        .map(|item| item.map(|(_, record)| record))
        .collect()
}

fn query_feegrant_records(
    deps: Deps,
    start_after: Option<Addr>,
    limit: Option<u32>,
) -> StdResult<Vec<FeeGrantRecord>> {
    let limit = limit.unwrap_or(30).min(100) as usize;
    let start = start_after.as_ref().map(Bound::exclusive);

    FEEGRANT_RECORDS
        .range(deps.storage, start, None, cosmwasm_std::Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, record)| record))
        .collect()
}

fn query_operator_consumption_records(
    deps: Deps,
    operator: Addr,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<ConsumptionRecord>> {
    let limit = limit.unwrap_or(30).min(100) as usize;
    let start = start_after.map(|s| s + 1);

    CONSUMPTION_RECORDS
        .range(
            deps.storage,
            start.map(|s| Bound::exclusive(s)),
            None,
            cosmwasm_std::Order::Ascending,
        )
        .filter(|item| {
            if let Ok((_, record)) = item {
                record.operator == operator
            } else {
                false
            }
        })
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
            cosmwasm_std::Order::Ascending,
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
            cosmwasm_std::Order::Ascending,
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
    handle_maci_contract_reply(deps, env, msg)
}

fn handle_maci_contract_reply(
    deps: DepsMut,
    env: Env,
    msg: Reply,
) -> Result<Response, ContractError> {
    let maci_counter = msg.id;

    if let SubMsgResult::Ok(response) = msg.result {
        // Parse the contract address from the response
        let contract_addr = response
            .events
            .iter()
            .find(|event| event.ty == "instantiate")
            .and_then(|event| {
                event
                    .attributes
                    .iter()
                    .find(|attr| attr.key == "_contract_address")
                    .map(|attr| attr.value.clone())
            })
            .ok_or(ContractError::ContractInstantiationFailed {})?;

        let contract_address = deps.api.addr_validate(&contract_addr)?;

        // We need to get operator info from the last consumption record
        // Since we just created a consumption record in execute_create_maci_round
        let consumption_counter = CONSUMPTION_COUNTER.load(deps.storage)?;
        let consumption_record = CONSUMPTION_RECORDS.load(deps.storage, consumption_counter)?;

        // Get round title from the consumption record description
        let round_title = consumption_record
            .description
            .split("'")
            .nth(1)
            .unwrap_or("Unknown Round")
            .to_string();

        // Create and save the MACI contract info
        let maci_info = MaciContractInfo {
            contract_address: contract_address.clone(),
            creator_operator: consumption_record.operator.clone(),
            round_title,
            created_at: env.block.time,
            code_id: 0, // We'll need to get this from somewhere else or store it differently
            creation_fee: consumption_record.amount,
        };

        MACI_CONTRACTS.save(deps.storage, maci_counter, &maci_info)?;
        MACI_CONTRACTS_BY_OPERATOR.save(
            deps.storage,
            (&consumption_record.operator, maci_counter),
            &true,
        )?;

        Ok(Response::new()
            .add_attribute("action", "maci_contract_created")
            .add_attribute("contract_address", contract_address.to_string())
            .add_attribute("creator", consumption_record.operator.to_string())
            .add_attribute("maci_id", maci_counter.to_string()))
    } else {
        Err(ContractError::ContractInstantiationFailed {})
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
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
