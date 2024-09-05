#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_json_binary, Addr, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, StdResult, SubMsg, SubMsgResponse, Uint128, Uint256, WasmMsg,
};

use cw2::set_contract_version;
use cw_amaci::contract::CREATED_ROUND_REPLY_ID;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, InstantiationData, QueryMsg};
use crate::state::{
    Admin, Config, ADMIN, CONFIG, MACI_DEACTIVATE_MESSAGE, MACI_DEACTIVATE_OPERATOR,
    MACI_OPERATOR_PUBKEY, MACI_OPERATOR_SET, OPERATOR, TOTAL,
};
use cw_amaci::msg::InstantiateMsg as AMaciInstantiateMsg;
use cw_amaci::state::{MaciParameters, PubKey, RoundInfo, VotingTime, Whitelist};
use cw_utils::parse_instantiate_response_data;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-amaci-registry";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin = Admin { admin: msg.admin };

    ADMIN.save(deps.storage, &admin)?;
    OPERATOR.save(deps.storage, &msg.operator)?;

    let config = Config {
        // denom: String::from("peaka"),
        denom: msg.denom,
        min_deposit_amount: msg.min_deposit_amount,
        slash_amount: msg.slash_amount,
    };
    CONFIG.save(deps.storage, &config)?;
    TOTAL.save(deps.storage, &0)?;

    Ok(Response::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Register { pubkey } => execute_register(deps, env, info, pubkey),
        ExecuteMsg::Deregister {} => execute_deregister(deps, env, info),
        ExecuteMsg::CreateRound {
            amaci_code_id,
            operator,
            max_voter,
            max_option,
            voice_credit_amount,
            round_info,
            voting_time,
            whitelist,
            pre_deactivate_root,
        } => execute_create_round(
            deps,
            env,
            info,
            amaci_code_id,
            operator,
            max_voter,
            max_option,
            voice_credit_amount,
            round_info,
            voting_time,
            whitelist,
            pre_deactivate_root,
        ),
        // ExecuteMsg::UploadDeactivateMessage {
        //     contract_address,
        //     deactivate_message,
        // } => {
        //     execute_upload_deactivate_message(deps, env, info, contract_address, deactivate_message)
        // }
        ExecuteMsg::ChangeParams {
            min_deposit_amount,
            slash_amount,
        } => execute_change_params(deps, env, info, min_deposit_amount, slash_amount),
        ExecuteMsg::ChangeOperator { address } => execute_change_operator(deps, env, info, address),
    }
}

pub fn execute_register(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    pubkey: PubKey, // amount: Uint128,
) -> Result<Response, ContractError> {
    if is_operator_set(deps.as_ref(), &info.sender)? {
        Err(ContractError::ExistedMaciOperator {})
    } else {
        let cfg = CONFIG.load(deps.storage)?;

        let denom = cfg.denom;
        let mut amount: Uint128 = Uint128::new(0);
        // Iterate through the funds and find the amount with the MACI denomination
        info.funds.iter().for_each(|fund| {
            if fund.denom == denom {
                amount = fund.amount;
            }
        });

        if amount < cfg.min_deposit_amount {
            return Err(ContractError::InsufficientDeposit {
                min_deposit_amount: cfg.min_deposit_amount,
            });
        }

        // update total
        let total = TOTAL.load(deps.storage)?;
        let new_total = total + amount.u128();
        TOTAL.save(deps.storage, &new_total)?;

        MACI_OPERATOR_SET.save(deps.storage, &info.sender, &amount)?;
        MACI_OPERATOR_PUBKEY.save(deps.storage, &info.sender, &pubkey)?;
        Ok(Response::new()
            .add_attribute("action", "register")
            .add_attribute("maci_operator", &info.sender.to_string())
            .add_attribute("amount", amount.to_string()))
    }
}

pub fn execute_deregister(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    if !is_operator_set(deps.as_ref(), &info.sender)? {
        Err(ContractError::Unauthorized {})
    } else {
        let operator_bond_amount = MACI_OPERATOR_SET.load(deps.storage, &info.sender)?;
        let cfg = CONFIG.load(deps.storage)?;
        let denom = cfg.denom;
        let amount_res = coins(operator_bond_amount.u128(), denom);
        let message = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: amount_res,
        };

        // update total
        let total = TOTAL.load(deps.storage)?;
        let new_total = total - operator_bond_amount.u128();
        TOTAL.save(deps.storage, &new_total)?;
        MACI_OPERATOR_SET.remove(deps.storage, &info.sender);

        Ok(Response::new()
            .add_message(message)
            .add_attribute("action", "deregister")
            .add_attribute("maci_operator", &info.sender.to_string())
            .add_attribute("amount", &operator_bond_amount.to_string()))
    }
}

pub fn execute_create_round(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amaci_code_id: u64,
    operator: Addr,
    max_voter: Uint256,
    max_option: Uint256,
    voice_credit_amount: Uint256,
    round_info: RoundInfo,
    voting_time: VotingTime,
    whitelist: Option<Whitelist>,
    pre_deactivate_root: Uint256,
) -> Result<Response, ContractError> {
    let maci_parameters: MaciParameters;
    if max_voter <= Uint256::from_u128(25u128) && max_option <= Uint256::from_u128(5u128) {
        // state_tree_depth: 2
        // vote_option_tree_depth: 1
        maci_parameters = MaciParameters {
            state_tree_depth: Uint256::from_u128(2u128),
            int_state_tree_depth: Uint256::from_u128(1u128),
            vote_option_tree_depth: Uint256::from_u128(1u128),
            message_batch_size: Uint256::from_u128(5u128),
        }
    } else if max_voter <= Uint256::from_u128(625u128) && max_option <= Uint256::from_u128(25u128) {
        // state_tree_depth: 4
        // vote_option_tree_depth: 2
        maci_parameters = MaciParameters {
            state_tree_depth: Uint256::from_u128(4u128),
            int_state_tree_depth: Uint256::from_u128(2u128),
            vote_option_tree_depth: Uint256::from_u128(2u128),
            message_batch_size: Uint256::from_u128(25u128),
        }
        // } else if max_voter <= 15625 && max_option <= 125 {
    } else {
        return Err(ContractError::NoMatchedSizeCircuit {});
    }

    let operator_pubkey = MACI_OPERATOR_PUBKEY.load(deps.storage, &operator)?;

    let init_msg = AMaciInstantiateMsg {
        parameters: maci_parameters,
        coordinator: operator_pubkey,
        operator,
        admin: info.sender,
        max_vote_options: max_option,
        voice_credit_amount,
        round_info,
        voting_time,
        whitelist,
        pre_deactivate_root,
    };

    let msg = WasmMsg::Instantiate {
        admin: Some(env.contract.address.to_string()),
        code_id: amaci_code_id,
        msg: to_json_binary(&init_msg)?,
        funds: vec![],
        label: "AMACI".to_string(),
    };

    let msg = SubMsg::reply_on_success(msg, CREATED_ROUND_REPLY_ID);

    let resp = Response::new()
        .add_submessage(msg)
        // .add_message(msg)
        .add_attribute("action", "create_maci_round")
        .add_attribute("amaci_code_id", &amaci_code_id.to_string());

    Ok(resp)
}

pub fn execute_change_params(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    min_deposit_amount: Uint128,
    slash_amount: Uint128,
) -> Result<Response, ContractError> {
    if !is_operator(deps.as_ref(), info.sender.as_ref())? {
        Err(ContractError::Unauthorized {})
    } else {
        let mut cfg = CONFIG.load(deps.storage)?;
        cfg.min_deposit_amount = min_deposit_amount;
        cfg.slash_amount = slash_amount;
        CONFIG.save(deps.storage, &cfg)?;
        Ok(Response::new()
            .add_attribute("action", "change_params")
            .add_attribute("min_deposit_amount", &min_deposit_amount.to_string())
            .add_attribute("slash_amount", &slash_amount.to_string()))
    }
}

pub fn execute_change_operator(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: Addr,
) -> Result<Response, ContractError> {
    if !is_admin(deps.as_ref(), info.sender.as_ref())? {
        Err(ContractError::Unauthorized {})
    } else {
        OPERATOR.save(deps.storage, &address)?;

        Ok(Response::new()
            .add_attribute("action", "change_operator")
            .add_attribute("address", address.to_string()))
    }
}

// pub fn execute_upload_deactivate_message(
//     deps: DepsMut,
//     _env: Env,
//     info: MessageInfo,
//     contract_address: Addr,
//     deactivate_message: Vec<Vec<Uint256>>,
// ) -> Result<Response, ContractError> {
//     if !is_operator_set(deps.as_ref(), &info.sender)? {
//         Err(ContractError::Unauthorized {})
//     } else {
//         let deactivate_format_data: Vec<Vec<String>> = deactivate_message
//             .iter()
//             .map(|input| input.iter().map(|f| f.to_string()).collect())
//             .collect();
//         MACI_DEACTIVATE_MESSAGE.save(deps.storage, &contract_address, &deactivate_format_data)?;
//         MACI_DEACTIVATE_OPERATOR.save(deps.storage, &contract_address, &info.sender)?;

//         Ok(Response::new()
//             .add_attribute("action", "upload_deactivate_message")
//             .add_attribute("contract_address", &contract_address.to_string())
//             .add_attribute("maci_operator", &info.sender.to_string())
//             .add_attribute(
//                 "deactivate_message",
//                 format!("{:?}", deactivate_format_data),
//             ))
//     }
// }

// Only admin can execute
fn is_admin(deps: Deps, sender: &str) -> StdResult<bool> {
    let cfg = ADMIN.load(deps.storage)?;
    let can = cfg.is_admin(&sender);
    Ok(can)
}

// Only operator/admin can execute
fn is_operator(deps: Deps, sender: &str) -> StdResult<bool> {
    let admin = ADMIN.load(deps.storage)?;
    let can_admin = admin.is_admin(&sender);

    let operator = OPERATOR.load(deps.storage)?;
    let can_operator = sender.to_string() == operator.to_string();

    let can = can_admin || can_operator;
    Ok(can)
}

fn is_operator_set(deps: Deps, sender: &Addr) -> StdResult<bool> {
    let res = MACI_OPERATOR_SET.has(deps.storage, sender);
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetTotal {} => to_json_binary(&TOTAL.load(deps.storage)?),
        QueryMsg::Admin {} => to_json_binary(&ADMIN.load(deps.storage)?),
        QueryMsg::Operator {} => to_json_binary(&OPERATOR.load(deps.storage)?),
        QueryMsg::IsMaciOperator { address } => {
            to_json_binary(&MACI_OPERATOR_SET.has(deps.storage, &address))
        }
        QueryMsg::GetConfig {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::GetMaciDeactivate { contract_address } => {
            to_json_binary(&MACI_DEACTIVATE_MESSAGE.load(deps.storage, &contract_address)?)
        }
        QueryMsg::GetMaciOperator { contract_address } => {
            to_json_binary(&MACI_DEACTIVATE_OPERATOR.load(deps.storage, &contract_address)?)
        }
        QueryMsg::GetMaciOperatorPubkey { address } => {
            to_json_binary(&MACI_OPERATOR_PUBKEY.load(deps.storage, &address)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        CREATED_ROUND_REPLY_ID => reply_created_round(deps, env, reply.result.into_result()),
        id => Err(ContractError::UnRecognizedReplyIdErr { id }),
    }
}

pub fn reply_created_round(
    deps: DepsMut,
    _env: Env,
    reply: Result<SubMsgResponse, String>,
) -> Result<Response, ContractError> {
    let response = reply.map_err(StdError::generic_err)?;
    let data = response.data.ok_or(ContractError::DataMissingErr {})?;
    // let response = parse_instantiate_response_data(&data)?;
    let response = match parse_instantiate_response_data(&data) {
        Ok(data) => data,
        Err(err) => {
            return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
                err.to_string(),
            )))
        }
    };

    let addr = Addr::unchecked(response.contract_address);
    let data = InstantiationData { addr: addr.clone() };
    let resp = Response::new()
        .add_attribute("action", "created_round")
        .add_attribute("round_addr", addr.to_string())
        .set_data(to_json_binary(&data)?);

    Ok(resp)
}
