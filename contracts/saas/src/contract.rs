#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, SubMsg, Uint128, Uint256, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::may_pay;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{
    Config, ConsumptionRecord, FeeGrantRecord, OperatorInfo, CONFIG, CONSUMPTION_COUNTER,
    CONSUMPTION_RECORDS, FEEGRANT_RECORDS, OPERATORS, TOTAL_BALANCE,
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
        ExecuteMsg::RemoveOperator { operator } => execute_remove_operator(deps, info, operator),
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

    Ok(Response::new()
        .add_attribute("method", "add_operator")
        .add_attribute("operator", operator.to_string()))
}

pub fn execute_remove_operator(
    deps: DepsMut,
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
