use crate::circuit_params::match_vkeys;
use crate::error::ContractError;
use crate::groth16_parser::{parse_groth16_proof, parse_groth16_vkey};
use crate::msg::{
    ExecuteMsg, Groth16ProofType, InstantiateMsg, InstantiationData, QueryMsg, WhitelistBase,
};
use crate::state::{
    Admin, DelayRecord, DelayRecords, DelayType, Groth16ProofStr, MessageData, Period,
    PeriodStatus, PubKey, QuinaryTreeRoot, RoundInfo, StateLeaf, VotingTime, Whitelist,
    WhitelistConfig, ADMIN, CERTSYSTEM, CIRCUITTYPE, COORDINATORHASH, CREATE_ROUND_WINDOW,
    CURRENT_DEACTIVATE_COMMITMENT, CURRENT_STATE_COMMITMENT, CURRENT_TALLY_COMMITMENT,
    DEACTIVATE_TIMEOUT, DELAY_RECORDS, DMSG_CHAIN_LENGTH, DMSG_HASHES, DNODES, FEEGRANTS,
    FIRST_DMSG_TIMESTAMP, GROTH16_DEACTIVATE_VKEYS, GROTH16_NEWKEY_VKEYS, GROTH16_PROCESS_VKEYS,
    GROTH16_TALLY_VKEYS, LEAF_IDX_0, MACIPARAMETERS, MACI_DEACTIVATE_MESSAGE, MACI_OPERATOR,
    MAX_LEAVES_COUNT, MAX_VOTE_OPTIONS, MSG_CHAIN_LENGTH, MSG_HASHES, NODES, NULLIFIERS,
    NUMSIGNUPS, PENALTY_RATE, PERIOD, PRE_DEACTIVATE_ROOT, PROCESSED_DMSG_COUNT,
    PROCESSED_MSG_COUNT, PROCESSED_USER_COUNT, QTR_LIB, RESULT, ROUNDINFO, SIGNUPED, STATEIDXINC,
    STATE_ROOT_BY_DMSG, TALLY_TIMEOUT, TOTAL_RESULT, VOICECREDITBALANCE, VOICE_CREDIT_AMOUNT,
    VOTEOPTIONMAP, VOTINGTIME, WHITELIST, ZEROS, ZEROS_H10,
};
use cosmwasm_schema::cw_serde;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;

use pairing_ce::bn256::Bn256;

use cosmos_sdk_proto::cosmos::base::v1beta1::Coin as SdkCoin;
use cosmos_sdk_proto::cosmos::feegrant::v1beta1::{
    AllowedMsgAllowance, BasicAllowance, MsgGrantAllowance, MsgRevokeAllowance,
};
use cosmos_sdk_proto::prost::Message;
use cosmos_sdk_proto::traits::TypeUrl;
use cosmos_sdk_proto::Any;
use prost_types::Timestamp as SdkTimestamp;

use crate::utils::{hash2, hash5, hash_256_uint256_list, uint256_from_hex_string};
use cosmwasm_std::{
    attr, coins, to_json_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Timestamp, Uint128, Uint256,
};

use bellman_ce_verifier::{prepare_verifying_key, verify_proof as groth16_verify};

use ff_ce::PrimeField as Fr;

use hex;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-amaci";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // Create an admin with the sender address
    let admin = Admin {
        admin: msg.admin.clone(),
    };
    ADMIN.save(deps.storage, &admin)?;

    // An error will be thrown if the number of vote options exceeds the circuit's capacity.
    let vote_option_max_amount = Uint256::from_u128(
        5u128.pow(
            msg.parameters
                .vote_option_tree_depth
                .to_string()
                .parse()
                .unwrap(),
        ),
    );
    if msg.max_vote_options > vote_option_max_amount {
        return Err(ContractError::MaxVoteOptionsExceeded {
            current: msg.max_vote_options,
            max_allowed: vote_option_max_amount,
        });
    }

    // if msg.voting_time.start_time >= msg.voting_time.end_time {
    //     return Err(ContractError::WrongTimeSet {});
    // }

    let create_round_window = Timestamp::from_seconds(10 * 60); // 10 minutes
    CREATE_ROUND_WINDOW.save(deps.storage, &create_round_window)?;

    // TODO: check apart time.
    if msg
        .voting_time
        .start_time
        .plus_seconds(create_round_window.seconds())
        >= msg.voting_time.end_time
    {
        return Err(ContractError::WrongTimeSet {});
    }

    match msg.whitelist {
        Some(content) => {
            let max_voter_amount = Uint256::from_u128(
                5u128.pow(msg.parameters.state_tree_depth.to_string().parse().unwrap()),
            );
            if Uint256::from_u128(content.users.len() as u128) > max_voter_amount {
                return Err(ContractError::MaxVoterExceeded {
                    current: Uint256::from_u128(content.users.len() as u128),
                    max_allowed: max_voter_amount,
                });
            }

            let mut users: Vec<WhitelistConfig> = Vec::new();
            for i in 0..content.users.len() {
                let data = WhitelistConfig {
                    addr: content.users[i].addr.clone(),
                    is_register: false,
                };
                users.push(data);
            }

            let whitelists = Whitelist { users };
            WHITELIST.save(deps.storage, &whitelists)?;
        }
        None => {}
    }

    // Save the MACI parameters to storage
    MACIPARAMETERS.save(deps.storage, &msg.parameters)?;
    let qtr_lab = QuinaryTreeRoot {
        zeros: [
            Uint256::from_u128(0u128),
            uint256_from_hex_string(
                "2066be41bebe6caf7e079360abe14fbf9118c62eabc42e2fe75e342b160a95bc",
            ),
            uint256_from_hex_string(
                "2a956d37d8e73692877b104630a08cc6840036f235f2134b0606769a369d85c1",
            ),
            uint256_from_hex_string(
                "2f9791ba036a4148ff026c074e713a4824415530dec0f0b16c5115aa00e4b825",
            ),
            uint256_from_hex_string(
                "2c41a7294c7ef5c9c5950dc627c55a00adb6712548bcbd6cd8569b1f2e5acc2a",
            ),
            uint256_from_hex_string(
                "2594ba68eb0f314eabbeea1d847374cc2be7965944dec513746606a1f2fadf2e",
            ),
            uint256_from_hex_string(
                "5c697158c9032bfd7041223a7dba696396388129118ae8f867266eb64fe7636",
            ),
            uint256_from_hex_string(
                "272b3425fcc3b2c45015559b9941fde27527aab5226045bf9b0a6c1fe902d601",
            ),
            uint256_from_hex_string(
                "268d82cc07023a1d5e7c987cbd0328b34762c9ea21369bea418f08b71b16846a",
            ),
        ],
    };

    // Save the qtr_lib value to storage
    QTR_LIB.save(deps.storage, &qtr_lab)?;

    // Save the pre_deactivate_root value to storage
    PRE_DEACTIVATE_ROOT.save(deps.storage, &msg.pre_deactivate_root)?;

    let vkey = match_vkeys(&msg.parameters)?;

    GROTH16_PROCESS_VKEYS.save(deps.storage, &vkey.process_vkey)?;
    GROTH16_TALLY_VKEYS.save(deps.storage, &vkey.tally_vkey)?;
    GROTH16_DEACTIVATE_VKEYS.save(deps.storage, &vkey.deactivate_vkey)?;
    GROTH16_NEWKEY_VKEYS.save(deps.storage, &vkey.add_key_vkey)?;

    // Compute the coordinator hash from the coordinator values in the message
    let coordinator_hash = hash2([msg.coordinator.x, msg.coordinator.y]);
    COORDINATORHASH.save(deps.storage, &coordinator_hash)?;

    // Compute the maximum number of leaves based on the state tree depth
    let max_leaves_count =
        Uint256::from_u128(5u128.pow(msg.parameters.state_tree_depth.to_string().parse().unwrap()));
    MAX_LEAVES_COUNT.save(deps.storage, &max_leaves_count)?;

    // Calculate the index of the first leaf in the tree
    let leaf_idx0 = (max_leaves_count - Uint256::from_u128(1u128)) / Uint256::from_u128(4u128);
    LEAF_IDX_0.save(deps.storage, &leaf_idx0)?;

    // Define an array of zero values
    let zeros_h10: [Uint256; 7] = [
        uint256_from_hex_string("26318ec8cdeef483522c15e9b226314ae39b86cde2a430dabf6ed19791917c47"),
        //     "17275449213996161510934492606295966958609980169974699290756906233261208992839",
        uint256_from_hex_string("28413250bf1cc56fabffd2fa32b52624941da885248fd1e015319e02c02abaf2"),
        //     "18207706266780806924962529690397914300960241391319167935582599262189180861170",
        uint256_from_hex_string("16738da97527034e095ac32bfab88497ca73a7b310a2744ab43971e82215cb6d"),
        //     "10155047796084846065379877743510757035594500557216694906214808863463609584493",
        uint256_from_hex_string("28140849348769fde6e971eec1424a5a162873a3d8adcbfdfc188e9c9d25faa3"),
        //     "18127908072205049515869530689345374790252438412920611306083118152373728836259",
        uint256_from_hex_string("1a07af159d19f68ed2aed0df224dabcc2e2321595968769f7c9e26591377ed9a"),
        //     "11773710380932653545559747058052522704305757415195021025284143362529247620506",
        uint256_from_hex_string("205cd249acba8f95f2e32ed51fa9c3d8e6f0d021892225d3efa9cd84c8fc1cad"),
        //     "14638012437623529368951445143647110672059367053598285839401224214917416754349",
        uint256_from_hex_string("b21c625cd270e71c2ee266c939361515e690be27e26cfc852a30b24e83504b0"),
        //     "5035114852453394843899296226690566678263173670465782309520655898931824493744",
    ];
    ZEROS_H10.save(deps.storage, &zeros_h10)?;

    NODES.save(
        deps.storage,
        Uint256::from_u128(0u128).to_be_bytes().to_vec(),
        &Uint256::from_u128(0u128),
    )?;

    // Define an array of zero values
    let zeros: [Uint256; 8] = [
        Uint256::from_u128(0u128),
        uint256_from_hex_string("2066be41bebe6caf7e079360abe14fbf9118c62eabc42e2fe75e342b160a95bc"),
        //     "14655542659562014735865511769057053982292279840403315552050801315682099828156",
        uint256_from_hex_string("2a956d37d8e73692877b104630a08cc6840036f235f2134b0606769a369d85c1"),
        //     "19261153649140605024552417994922546473530072875902678653210025980873274131905",
        uint256_from_hex_string("2f9791ba036a4148ff026c074e713a4824415530dec0f0b16c5115aa00e4b825"),
        //     "21526503558325068664033192388586640128492121680588893182274749683522508994597",
        uint256_from_hex_string("2c41a7294c7ef5c9c5950dc627c55a00adb6712548bcbd6cd8569b1f2e5acc2a"),
        //     "20017764101928005973906869479218555869286328459998999367935018992260318153770",
        uint256_from_hex_string("2594ba68eb0f314eabbeea1d847374cc2be7965944dec513746606a1f2fadf2e"),
        //     "16998355316577652097112514691750893516081130026395813155204269482715045879598",
        uint256_from_hex_string("5c697158c9032bfd7041223a7dba696396388129118ae8f867266eb64fe7636"),
        //     "2612442706402737973181840577010736087708621987282725873936541279764292204086",
        uint256_from_hex_string("272b3425fcc3b2c45015559b9941fde27527aab5226045bf9b0a6c1fe902d601"),
        //     "17716535433480122581515618850811568065658392066947958324371350481921422579201",
        // uint256_from_hex_string("268d82cc07023a1d5e7c987cbd0328b34762c9ea21369bea418f08b71b16846a"),
        //     "17437916409890180001398333108882255895598851862997171508841759030332444017770",
    ];
    ZEROS.save(deps.storage, &zeros)?;

    // Save initial values for message hash, message chain length, processed message count, current tally commitment,
    // processed user count, and number of signups to storage
    MSG_HASHES.save(
        deps.storage,
        Uint256::from_u128(0u128).to_be_bytes().to_vec(),
        &Uint256::from_u128(0u128),
    )?;
    MSG_CHAIN_LENGTH.save(deps.storage, &Uint256::from_u128(0u128))?;
    PROCESSED_MSG_COUNT.save(deps.storage, &Uint256::from_u128(0u128))?;
    CURRENT_TALLY_COMMITMENT.save(deps.storage, &Uint256::from_u128(0u128))?;
    PROCESSED_USER_COUNT.save(deps.storage, &Uint256::from_u128(0u128))?;
    NUMSIGNUPS.save(deps.storage, &Uint256::from_u128(0u128))?;
    MAX_VOTE_OPTIONS.save(deps.storage, &msg.max_vote_options)?;
    VOICE_CREDIT_AMOUNT.save(deps.storage, &msg.voice_credit_amount)?;

    PROCESSED_DMSG_COUNT.save(deps.storage, &Uint256::from_u128(0u128))?;
    DMSG_CHAIN_LENGTH.save(deps.storage, &Uint256::from_u128(0u128))?;

    let current_dcommitment = &hash2([
        zeros[msg
            .parameters
            .state_tree_depth
            .to_string()
            .parse::<usize>()
            .unwrap()],
        zeros[msg
            .parameters
            .state_tree_depth
            .to_string()
            .parse::<usize>()
            .unwrap()],
    ]);
    CURRENT_DEACTIVATE_COMMITMENT.save(deps.storage, current_dcommitment)?;
    DMSG_HASHES.save(
        deps.storage,
        Uint256::from_u128(0u128).to_be_bytes().to_vec(),
        &Uint256::from_u128(0u128),
    )?;
    STATE_ROOT_BY_DMSG.save(
        deps.storage,
        Uint256::from_u128(0u128).to_be_bytes().to_vec(),
        &Uint256::from_u128(0u128),
    )?;
    DNODES.save(
        deps.storage,
        Uint256::from_u128(0u128).to_be_bytes().to_vec(),
        &Uint256::from_u128(0u128),
    )?;

    let mut vote_option_map: Vec<String> = Vec::new();
    for _ in 0..msg.max_vote_options.to_string().parse().unwrap() {
        vote_option_map.push(String::new());
    }
    VOTEOPTIONMAP.save(deps.storage, &vote_option_map)?;
    ROUNDINFO.save(deps.storage, &msg.round_info)?;

    VOTINGTIME.save(deps.storage, &msg.voting_time)?;

    // Create a period struct with the initial status set to Voting
    let period = Period {
        status: PeriodStatus::Pending,
    };

    // Save the initial period to storage
    PERIOD.save(deps.storage, &period)?;

    MACI_OPERATOR.save(deps.storage, &msg.operator)?;

    let circuit_type = if msg.circuit_type == Uint256::from_u128(0u128) {
        "0" // 1p1v
    } else if msg.circuit_type == Uint256::from_u128(1u128) {
        "1" // qv
    } else {
        return Err(ContractError::UnsupportedCircuitType {});
    };

    let certification_system = if msg.certification_system == Uint256::from_u128(0u128) {
        "groth16" // groth16
    } else {
        return Err(ContractError::UnsupportedCertificationSystem {});
    };

    CIRCUITTYPE.save(deps.storage, &msg.circuit_type)?;
    CERTSYSTEM.save(deps.storage, &msg.certification_system)?;

    // Init penalty rate and timeout
    let penalty_rate = Uint256::from_u128(80);
    PENALTY_RATE.save(deps.storage, &penalty_rate)?; // 80%
                                                     // let deactivate_timeout = Timestamp::from_seconds(15 * 60); // 15 minutes
                                                     // let tally_timeout = Timestamp::from_seconds(1 * 3600); // 1 hour

    // let deactivate_timeout = Timestamp::from_seconds(5); // for test
    //     let tally_timeout = Timestamp::from_seconds(30); // for test

    let deactivate_timeout = Timestamp::from_seconds(5 * 60); // 5 minutes
    let tally_timeout = Timestamp::from_seconds(30 * 60); // 30 minutes
    DEACTIVATE_TIMEOUT.save(deps.storage, &deactivate_timeout)?;
    TALLY_TIMEOUT.save(deps.storage, &tally_timeout)?;
    DELAY_RECORDS.save(deps.storage, &DelayRecords { records: vec![] })?;

    let data: InstantiationData = InstantiationData {
        caller: info.sender.clone(),
        parameters: msg.parameters.clone(),
        coordinator: msg.coordinator.clone(),
        admin: msg.admin.clone(),
        operator: msg.operator.clone(),
        max_vote_options: msg.max_vote_options.clone(),
        voice_credit_amount: msg.voice_credit_amount.clone(),
        round_info: msg.round_info.clone(),
        voting_time: msg.voting_time.clone(),
        pre_deactivate_root: msg.pre_deactivate_root.clone(),
        circuit_type: circuit_type.to_string(),
        certification_system: certification_system.to_string(),
        penalty_rate: penalty_rate.clone(),
        deactivate_timeout: deactivate_timeout.clone(),
        tally_timeout: tally_timeout.clone(),
    };

    let mut attributes = vec![
        attr("action", "instantiate"),
        attr("caller", &info.sender.to_string()),
        attr("admin", &msg.admin.to_string()),
        attr("operator", &msg.operator.to_string()),
        attr(
            "voting_start",
            &msg.voting_time.start_time.nanos().to_string(),
        ),
        attr("voting_end", &msg.voting_time.end_time.nanos().to_string()),
        attr("round_title", &msg.round_info.title.to_string()),
        attr("coordinator_pubkey_x", &msg.coordinator.x.to_string()),
        attr("coordinator_pubkey_y", &msg.coordinator.y.to_string()),
        attr("max_vote_options", &msg.max_vote_options.to_string()),
        attr("voice_credit_amount", &msg.voice_credit_amount.to_string()),
        attr("pre_deactivate_root", &msg.pre_deactivate_root.to_string()),
        attr(
            "state_tree_depth",
            &msg.parameters.state_tree_depth.to_string(),
        ),
        attr(
            "int_state_tree_depth",
            &msg.parameters.int_state_tree_depth.to_string(),
        ),
        attr(
            "vote_option_tree_depth",
            &msg.parameters.vote_option_tree_depth.to_string(),
        ),
        attr(
            "message_batch_size",
            &msg.parameters.message_batch_size.to_string(),
        ),
        attr("circuit_type", &circuit_type.to_string()),
        attr("certification_system", &certification_system.to_string()),
        attr("penalty_rate", &penalty_rate.to_string()),
        attr(
            "deactivate_timeout",
            &deactivate_timeout.seconds().to_string(),
        ),
        attr("tally_timeout", &tally_timeout.seconds().to_string()),
    ];

    if msg.round_info.description != "" {
        attributes.push(attr("round_description", msg.round_info.description))
    }

    if msg.round_info.link != "" {
        attributes.push(attr("round_link", msg.round_info.link))
    }

    Ok(Response::new()
        .add_attributes(attributes)
        .set_data(to_json_binary(&data)?))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetRoundInfo { round_info } => {
            execute_set_round_info(deps, env, info, round_info)
        }
        ExecuteMsg::SetWhitelists { whitelists } => {
            execute_set_whitelists(deps, env, info, whitelists)
        }
        ExecuteMsg::SetVoteOptionsMap { vote_option_map } => {
            execute_set_vote_options_map(deps, env, info, vote_option_map)
        }
        // ExecuteMsg::StartVotingPeriod {} => execute_start_voting_period(deps, env, info),
        ExecuteMsg::SignUp { pubkey } => execute_sign_up(deps, env, info, pubkey),
        // ExecuteMsg::StopVotingPeriod {} => execute_stop_voting_period(deps, env, info),
        ExecuteMsg::PublishDeactivateMessage {
            message,
            enc_pub_key,
        } => execute_publish_deactivate_message(deps, env, info, message, enc_pub_key),
        ExecuteMsg::UploadDeactivateMessage { deactivate_message } => {
            execute_upload_deactivate_message(deps, env, info, deactivate_message)
        }
        ExecuteMsg::ProcessDeactivateMessage {
            size,
            new_deactivate_commitment,
            new_deactivate_root,
            groth16_proof,
        } => execute_process_deactivate_message(
            deps,
            env,
            info,
            size,
            new_deactivate_commitment,
            new_deactivate_root,
            groth16_proof,
        ),
        ExecuteMsg::AddNewKey {
            pubkey,
            nullifier,
            d,
            groth16_proof,
        } => execute_add_new_key(deps, env, info, pubkey, nullifier, d, groth16_proof),
        ExecuteMsg::PreAddNewKey {
            pubkey,
            nullifier,
            d,
            groth16_proof,
        } => execute_pre_add_new_key(deps, env, info, pubkey, nullifier, d, groth16_proof),
        ExecuteMsg::PublishMessage {
            message,
            enc_pub_key,
        } => execute_publish_message(deps, env, info, message, enc_pub_key),
        ExecuteMsg::StartProcessPeriod {} => execute_start_process_period(deps, env, info),
        ExecuteMsg::ProcessMessage {
            new_state_commitment,
            groth16_proof,
        } => execute_process_message(deps, env, info, new_state_commitment, groth16_proof),
        ExecuteMsg::StopProcessingPeriod {} => execute_stop_processing_period(deps, env, info),
        ExecuteMsg::ProcessTally {
            new_tally_commitment,
            groth16_proof,
        } => execute_process_tally(deps, env, info, new_tally_commitment, groth16_proof),
        ExecuteMsg::StopTallyingPeriod { results, salt } => {
            execute_stop_tallying_period(deps, env, info, results, salt)
        }
        ExecuteMsg::Grant { max_amount } => execute_grant(deps, env, info, max_amount),
        ExecuteMsg::Revoke {} => execute_revoke(deps, env, info),
        ExecuteMsg::Bond {} => execute_bond(deps, env, info),
        ExecuteMsg::Withdraw { amount } => execute_withdraw(deps, env, info, amount),
    }
}

pub fn execute_set_round_info(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    round_info: RoundInfo,
) -> Result<Response, ContractError> {
    if !is_admin(deps.as_ref(), info.sender.as_ref())? {
        Err(ContractError::Unauthorized {})
    } else {
        if round_info.title == "" {
            return Err(ContractError::TitleIsEmpty {});
        }

        ROUNDINFO.save(deps.storage, &round_info)?;

        let mut attributes = vec![attr("action", "set_round_info")];
        attributes.push(attr("title", round_info.title));

        if round_info.description != "" {
            attributes.push(attr("description", round_info.description))
        }

        if round_info.link != "" {
            attributes.push(attr("link", round_info.link))
        }

        Ok(Response::new().add_attributes(attributes))
    }
}

// in pending
pub fn execute_set_whitelists(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    whitelists: WhitelistBase,
) -> Result<Response, ContractError> {
    if FEEGRANTS.exists(deps.storage) {
        return Err(ContractError::FeeGrantAlreadyExists);
    }

    let voting_time = VOTINGTIME.load(deps.storage)?;

    if env.block.time >= voting_time.start_time {
        return Err(ContractError::PeriodError {});
    }

    if !is_admin(deps.as_ref(), info.sender.as_ref())? {
        Err(ContractError::Unauthorized {})
    } else {
        let cfg = MACIPARAMETERS.load(deps.storage)?;

        let max_voter_amount =
            Uint256::from_u128(5u128.pow(cfg.state_tree_depth.to_string().parse().unwrap()));
        if Uint256::from_u128(whitelists.users.len() as u128) > max_voter_amount {
            return Err(ContractError::MaxVoterExceeded {
                current: Uint256::from_u128(whitelists.users.len() as u128),
                max_allowed: max_voter_amount,
            });
        }

        let mut users: Vec<WhitelistConfig> = Vec::new();
        for i in 0..whitelists.users.len() {
            let data = WhitelistConfig {
                addr: whitelists.users[i].addr.clone(),
                is_register: false,
            };
            users.push(data);
        }

        let whitelists = Whitelist { users };
        WHITELIST.save(deps.storage, &whitelists)?;
        let res = Response::new().add_attribute("action", "set_whitelists");
        Ok(res)
    }
}

// in pending
pub fn execute_set_vote_options_map(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vote_option_map: Vec<String>,
) -> Result<Response, ContractError> {
    let voting_time = VOTINGTIME.load(deps.storage)?;

    if env.block.time >= voting_time.start_time {
        return Err(ContractError::PeriodError {});
    }

    if !is_admin(deps.as_ref(), info.sender.as_ref())? {
        Err(ContractError::Unauthorized {})
    } else {
        let max_vote_options = vote_option_map.len() as u128;
        let cfg = MACIPARAMETERS.load(deps.storage)?;

        // An error will be thrown if the number of vote options exceeds the circuit's capacity.
        let vote_option_max_amount =
            Uint256::from_u128(5u128.pow(cfg.vote_option_tree_depth.to_string().parse().unwrap()));
        if Uint256::from_u128(max_vote_options) > vote_option_max_amount {
            return Err(ContractError::MaxVoteOptionsExceeded {
                current: Uint256::from_u128(max_vote_options),
                max_allowed: vote_option_max_amount,
            });
        }

        VOTEOPTIONMAP.save(deps.storage, &vote_option_map)?;
        // Save the maximum vote options
        MAX_VOTE_OPTIONS.save(deps.storage, &Uint256::from_u128(max_vote_options))?;
        let res = Response::new()
            .add_attribute("action", "set_vote_option")
            .add_attribute("vote_option_map", format!("{:?}", vote_option_map))
            .add_attribute("max_vote_options", max_vote_options.to_string());
        Ok(res)
    }
}

// in voting
pub fn execute_sign_up(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pubkey: PubKey,
) -> Result<Response, ContractError> {
    let voting_time = VOTINGTIME.load(deps.storage)?;
    check_voting_time(env, voting_time)?;
    if !is_whitelist(deps.as_ref(), &info.sender)? {
        return Err(ContractError::Unauthorized {});
    }

    if is_register(deps.as_ref(), &info.sender)? {
        return Err(ContractError::UserAlreadyRegistered {});
    }
    // let user_balance = user_balance_of(deps.as_ref(), info.sender.as_ref())?;
    // if user_balance == Uint256::from_u128(0u128) {
    //     return Err(ContractError::Unauthorized {});
    // }
    let voice_credit_amount = VOICE_CREDIT_AMOUNT.load(deps.storage)?;

    let mut num_sign_ups = NUMSIGNUPS.load(deps.storage)?;

    let max_leaves_count = MAX_LEAVES_COUNT.load(deps.storage)?;

    // // Load the scalar field value
    let snark_scalar_field =
        uint256_from_hex_string("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");
    // let snark_scalar_field = uint256_from_decimal_string(
    // "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    // );

    // Check if the number of sign-ups is less than the maximum number of leaves
    assert!(num_sign_ups < max_leaves_count, "full");
    // Check if the pubkey values are within the allowed range
    assert!(
        pubkey.x < snark_scalar_field && pubkey.y < snark_scalar_field,
        "MACI: pubkey values should be less than the snark scalar field"
    );

    // Create a state leaf with the provided pubkey and amount
    let state_leaf = StateLeaf {
        pub_key: pubkey.clone(),
        voice_credit_balance: voice_credit_amount,
        vote_option_tree_root: Uint256::from_u128(0),
        nonce: Uint256::from_u128(0),
    }
    .hash_decativate_state_leaf();

    let state_index = num_sign_ups;
    // Enqueue the state leaf
    state_enqueue(&mut deps, state_leaf)?;
    num_sign_ups += Uint256::from_u128(1u128);

    // Save the updated state index, voice credit balance, and number of sign-ups
    // STATEIDXINC.save(deps.storage, &info.sender, &num_sign_ups)?;
    // VOICECREDITBALANCE.save(
    //     deps.storage,
    //     state_index.to_be_bytes().to_vec(),
    //     &voice_credit_amount,
    // )?;
    NUMSIGNUPS.save(deps.storage, &num_sign_ups)?;
    SIGNUPED.save(deps.storage, pubkey.x.to_be_bytes().to_vec(), &num_sign_ups)?;

    let mut whitelist = WHITELIST.load(deps.storage)?;
    whitelist.register(&info.sender);
    WHITELIST.save(deps.storage, &whitelist)?;

    Ok(Response::new()
        .add_attribute("action", "sign_up")
        .add_attribute("state_idx", state_index.to_string())
        .add_attribute(
            "pubkey",
            format!("{:?},{:?}", pubkey.x.to_string(), pubkey.y.to_string()),
        )
        .add_attribute("balance", voice_credit_amount.to_string()))
}

// in voting
pub fn execute_publish_message(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    message: MessageData,
    enc_pub_key: PubKey,
) -> Result<Response, ContractError> {
    // Check if the period status is Voting
    let voting_time = VOTINGTIME.load(deps.storage)?;
    check_voting_time(env, voting_time)?;

    // Load the scalar field value
    let snark_scalar_field =
        uint256_from_hex_string("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

    // let snark_scalar_field = uint256_from_decimal_string(
    //     "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    // );

    // Check if the encrypted public key is valid
    if enc_pub_key.x != Uint256::from_u128(0u128)
        && enc_pub_key.y != Uint256::from_u128(1u128)
        && enc_pub_key.x < snark_scalar_field
        && enc_pub_key.y < snark_scalar_field
    {
        let mut msg_chain_length = MSG_CHAIN_LENGTH.load(deps.storage)?;
        let old_msg_hashes =
            MSG_HASHES.load(deps.storage, msg_chain_length.to_be_bytes().to_vec())?;

        // Compute the new message hash using the provided message, encrypted public key, and previous hash
        MSG_HASHES.save(
            deps.storage,
            (msg_chain_length + Uint256::from_u128(1u128))
                .to_be_bytes()
                .to_vec(),
            &hash_message_and_enc_pub_key(message.clone(), enc_pub_key.clone(), old_msg_hashes),
        )?;

        let old_chain_length = msg_chain_length;
        // Update the message chain length
        msg_chain_length += Uint256::from_u128(1u128);
        MSG_CHAIN_LENGTH.save(deps.storage, &msg_chain_length)?;
        // Return a success response
        Ok(Response::new()
            .add_attribute("action", "publish_message")
            .add_attribute("msg_chain_length", old_chain_length.to_string())
            .add_attribute("message", format!("{:?}", message.data))
            .add_attribute(
                "enc_pub_key",
                format!(
                    "{:?},{:?}",
                    enc_pub_key.x.to_string(),
                    enc_pub_key.y.to_string()
                ),
            ))
    } else {
        // Return an error response for invalid user or encrypted public key
        Ok(Response::new()
            .add_attribute("action", "publish_message")
            .add_attribute("event", "error user."))
    }
}

// in voting
pub fn execute_publish_deactivate_message(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    message: MessageData,
    enc_pub_key: PubKey,
) -> Result<Response, ContractError> {
    // Check if the period status is Voting
    let voting_time = VOTINGTIME.load(deps.storage)?;
    check_voting_time(env.clone(), voting_time)?;

    // Load the scalar field value
    let snark_scalar_field =
        uint256_from_hex_string("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

    // let snark_scalar_field = uint256_from_decimal_string(
    //     "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    // );
    // Check if the encrypted public key is valid
    if enc_pub_key.x != Uint256::from_u128(0u128)
        && enc_pub_key.y != Uint256::from_u128(1u128)
        && enc_pub_key.x < snark_scalar_field
        && enc_pub_key.y < snark_scalar_field
    {
        let processed_dmsg_count = PROCESSED_DMSG_COUNT.load(deps.storage)?;
        let mut dmsg_chain_length = DMSG_CHAIN_LENGTH.load(deps.storage)?;

        // When the processed_dmsg_count catches up with dmsg_chain_length, it indicates that the previous batch has been processed.
        // At this point, the new incoming message is the first one of the new batch, and we record the timestamp.
        if processed_dmsg_count == dmsg_chain_length {
            FIRST_DMSG_TIMESTAMP.save(deps.storage, &env.block.time)?;
        }

        let old_msg_hashes =
            DMSG_HASHES.load(deps.storage, dmsg_chain_length.to_be_bytes().to_vec())?;

        let mut m: [Uint256; 5] = [Uint256::zero(); 5];
        m[0] = message.data[0];
        m[1] = message.data[1];
        m[2] = message.data[2];
        m[3] = message.data[3];
        m[4] = message.data[4];

        let mut n: [Uint256; 5] = [Uint256::zero(); 5];
        n[0] = message.data[5];
        n[1] = message.data[6];
        n[2] = enc_pub_key.x;
        n[3] = enc_pub_key.y;
        n[4] = old_msg_hashes;

        let m_hash = hash5(m);

        let n_hash = hash5(n);
        let m_n_hash = hash2([m_hash, n_hash]);

        // Compute the new message hash using the provided message, encrypted public key, and previous hash
        DMSG_HASHES.save(
            deps.storage,
            (dmsg_chain_length + Uint256::from_u128(1u128))
                .to_be_bytes()
                .to_vec(),
            &m_n_hash,
        )?;

        let state_root = state_root(deps.as_ref());

        STATE_ROOT_BY_DMSG.save(
            deps.storage,
            (dmsg_chain_length + Uint256::from_u128(1u128))
                .to_be_bytes()
                .to_vec(),
            &state_root,
        )?;

        let old_chain_length = dmsg_chain_length;
        // Update the message chain length
        dmsg_chain_length += Uint256::from_u128(1u128);
        DMSG_CHAIN_LENGTH.save(deps.storage, &dmsg_chain_length)?;

        let num_sign_ups = NUMSIGNUPS.load(deps.storage)?;

        Ok(Response::new()
            .add_attribute("action", "publish_deactivate_message")
            .add_attribute("dmsg_chain_length", old_chain_length.to_string())
            .add_attribute("num_sign_ups", num_sign_ups.to_string())
            .add_attribute("message", format!("{:?}", message.data))
            .add_attribute(
                "enc_pub_key",
                format!(
                    "{:?},{:?}",
                    enc_pub_key.x.to_string(),
                    enc_pub_key.y.to_string()
                ),
            ))
    } else {
        // Return an error response for invalid user or encrypted public key
        Ok(Response::new()
            .add_attribute("action", "publish_deactivate_message")
            .add_attribute("event", "error user."))
    }
}

pub fn execute_upload_deactivate_message(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    deactivate_message: Vec<Vec<Uint256>>,
) -> Result<Response, ContractError> {
    if !is_operator(deps.as_ref(), &info.sender.as_ref())? {
        Err(ContractError::Unauthorized {})
    } else {
        let deactivate_format_data: Vec<Vec<String>> = deactivate_message
            .iter()
            .map(|input| input.iter().map(|f| f.to_string()).collect())
            .collect();
        MACI_DEACTIVATE_MESSAGE.save(
            deps.storage,
            &env.contract.address,
            &deactivate_format_data,
        )?;
        // MACI_DEACTIVATE_OPERATOR.save(deps.storage, &contract_address, &info.sender)?;

        Ok(Response::new()
            .add_attribute("action", "upload_deactivate_message")
            .add_attribute("contract_address", &env.contract.address.to_string())
            .add_attribute("maci_operator", &info.sender.to_string())
            .add_attribute(
                "deactivate_message",
                format!("{:?}", deactivate_format_data),
            ))
    }
}

// all time
pub fn execute_process_deactivate_message(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    size: Uint256,
    new_deactivate_commitment: Uint256,
    new_deactivate_root: Uint256,
    groth16_proof: Groth16ProofType,
) -> Result<Response, ContractError> {
    // // Check if the period status is Voting
    // let voting_time = VOTINGTIME.load(deps.storage)?;
    // check_voting_time(env, voting_time)?;

    let processed_dmsg_count = PROCESSED_DMSG_COUNT.load(deps.storage)?;
    let dmsg_chain_length = DMSG_CHAIN_LENGTH.load(deps.storage)?;

    assert!(
        processed_dmsg_count < dmsg_chain_length,
        "all deactivate messages have been processed"
    );

    // Load the MACI parameters
    let parameters = MACIPARAMETERS.load(deps.storage)?;
    let batch_size = parameters.message_batch_size;

    assert!(size <= batch_size, "size overflow the batchsize");

    DNODES.save(
        deps.storage,
        Uint256::from_u128(0u128).to_be_bytes().to_vec(),
        &new_deactivate_root,
    )?;
    let mut input: [Uint256; 7] = [Uint256::zero(); 7];
    input[0] = new_deactivate_root;
    input[1] = COORDINATORHASH.load(deps.storage)?;
    let batch_start_index = processed_dmsg_count;
    let mut batch_end_index = batch_start_index + size;
    let dmsg_chain_length = DMSG_CHAIN_LENGTH.load(deps.storage)?;
    if batch_end_index > dmsg_chain_length {
        batch_end_index = dmsg_chain_length;
    }

    input[2] = DMSG_HASHES.load(deps.storage, batch_start_index.to_be_bytes().to_vec())?;
    input[3] = DMSG_HASHES.load(deps.storage, batch_end_index.to_be_bytes().to_vec())?;

    input[4] = CURRENT_DEACTIVATE_COMMITMENT.load(deps.storage)?;
    input[5] = new_deactivate_commitment;
    input[6] = STATE_ROOT_BY_DMSG.load(deps.storage, batch_end_index.to_be_bytes().to_vec())?;

    // Load the scalar field value
    let snark_scalar_field =
        uint256_from_hex_string("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");
    // let snark_scalar_field = uint256_from_decimal_string(
    //     "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    // );

    // Compute the hash of the input values
    let input_hash = uint256_from_hex_string(&hash_256_uint256_list(&input)) % snark_scalar_field;

    // Load the process verification keys
    let deactivate_vkeys_str = GROTH16_DEACTIVATE_VKEYS.load(deps.storage)?;

    // Parse the SNARK proof
    let proof_str = Groth16ProofStr {
        pi_a: hex::decode(groth16_proof.a.clone())
            .map_err(|_| ContractError::HexDecodingError {})?,
        pi_b: hex::decode(groth16_proof.b.clone())
            .map_err(|_| ContractError::HexDecodingError {})?,
        pi_c: hex::decode(groth16_proof.c.clone())
            .map_err(|_| ContractError::HexDecodingError {})?,
    };

    // Parse the verification key and prepare for verification
    let vkey = parse_groth16_vkey::<Bn256>(deactivate_vkeys_str)?;
    let pvk = prepare_verifying_key(&vkey);

    // Parse the proof and prepare for verification
    let pof = parse_groth16_proof::<Bn256>(proof_str.clone())?;

    // Verify the SNARK proof using the input hash
    let is_passed = groth16_verify(
        &pvk,
        &pof,
        &[Fr::from_str(&input_hash.to_string()).unwrap()],
    )
    .unwrap();

    // If the proof verification fails, return an error
    if !is_passed {
        return Err(ContractError::InvalidProof {
            step: String::from("ProcessDeactivate"),
        });
    }

    CURRENT_DEACTIVATE_COMMITMENT.save(deps.storage, &new_deactivate_commitment)?;
    PROCESSED_DMSG_COUNT.save(
        deps.storage,
        &(processed_dmsg_count + batch_end_index - batch_start_index),
    )?;
    let mut attributes = vec![
        attr("zk_verify", is_passed.to_string()),
        attr("commitment", new_deactivate_commitment.to_string()),
        attr("proof", format!("{:?}", groth16_proof)),
        attr("certification_system", "groth16"),
        attr("processed_dmsg_count", processed_dmsg_count.to_string()),
    ];

    let first_dmsg_time: Timestamp = FIRST_DMSG_TIMESTAMP.load(deps.storage)?;
    let current_time = env.block.time;

    let different_time: u64 = current_time.seconds() - first_dmsg_time.seconds();

    if different_time > DEACTIVATE_TIMEOUT.load(deps.storage)?.seconds() {
        let mut delay_records = DELAY_RECORDS.load(deps.storage)?;
        let delay_timestamp = first_dmsg_time;
        let delay_duration = different_time;
        let delay_reason = format!(
            "Processing of {} deactivate messages has timed out after {} seconds",
            size, different_time
        );
        let delay_process_dmsg_count = batch_end_index - batch_start_index;
        let delay_type = DelayType::DeactivateDelay;
        let delay_record = DelayRecord {
            delay_timestamp: delay_timestamp.clone(),
            delay_duration: delay_duration.clone(),
            delay_reason: delay_reason.clone(),
            delay_process_dmsg_count: delay_process_dmsg_count.clone(),
            delay_type,
        };
        delay_records.records.push(delay_record);
        DELAY_RECORDS.save(deps.storage, &delay_records)?;
        attributes.push(attr(
            "delay_timestamp",
            delay_timestamp.seconds().to_string(),
        ));
        attributes.push(attr("delay_duration", delay_duration.to_string()));
        attributes.push(attr(
            "delay_process_dmsg_count",
            delay_process_dmsg_count.to_string(),
        ));
        attributes.push(attr("delay_reason", delay_reason));

        attributes.push(attr("delay_type", "deactivate_delay"));
    }

    Ok(Response::new()
        .add_attribute("action", "process_deactivate_message")
        .add_attributes(attributes))
}

// in voting
pub fn execute_add_new_key(
    mut deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    pubkey: PubKey,
    nullifier: Uint256,
    d: [Uint256; 4],
    groth16_proof: Groth16ProofType,
) -> Result<Response, ContractError> {
    // Check if the period status is Voting
    let voting_time = VOTINGTIME.load(deps.storage)?;
    check_voting_time(env, voting_time)?;

    if NULLIFIERS.has(deps.storage, nullifier.to_be_bytes().to_vec()) {
        // Return an error response for invalid user or encrypted public key
        return Err(ContractError::NewKeyExist {});
    }

    NULLIFIERS.save(deps.storage, nullifier.to_be_bytes().to_vec(), &true)?;

    let mut num_sign_ups = NUMSIGNUPS.load(deps.storage)?;

    let max_leaves_count = MAX_LEAVES_COUNT.load(deps.storage)?;

    // // Load the scalar field value
    let snark_scalar_field =
        uint256_from_hex_string("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

    assert!(num_sign_ups < max_leaves_count, "full");
    // Check if the pubkey values are within the allowed range
    assert!(
        pubkey.x < snark_scalar_field && pubkey.y < snark_scalar_field,
        "MACI: pubkey values should be less than the snark scalar field"
    );

    let mut input: [Uint256; 7] = [Uint256::zero(); 7];
    input[0] = DNODES.load(
        deps.storage,
        Uint256::from_u128(0u128).to_be_bytes().to_vec(),
    )?;
    input[1] = COORDINATORHASH.load(deps.storage)?;
    input[2] = nullifier;
    input[3] = d[0];
    input[4] = d[1];
    input[5] = d[2];
    input[6] = d[3];

    // Load the scalar field value
    let snark_scalar_field =
        uint256_from_hex_string("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");
    //     "21888242871839275222246405745257275088548364400416034343698204186575808495617",

    // Compute the hash of the input values
    let input_hash = uint256_from_hex_string(&hash_256_uint256_list(&input)) % snark_scalar_field; // input hash

    // Load the process verification keys
    let process_vkeys_str = GROTH16_NEWKEY_VKEYS.load(deps.storage)?;

    // Parse the SNARK proof
    let proof_str = Groth16ProofStr {
        pi_a: hex::decode(groth16_proof.a.clone())
            .map_err(|_| ContractError::HexDecodingError {})?,
        pi_b: hex::decode(groth16_proof.b.clone())
            .map_err(|_| ContractError::HexDecodingError {})?,
        pi_c: hex::decode(groth16_proof.c.clone())
            .map_err(|_| ContractError::HexDecodingError {})?,
    };

    // Parse the verification key and prepare for verification
    let vkey = parse_groth16_vkey::<Bn256>(process_vkeys_str)?;
    let pvk = prepare_verifying_key(&vkey);

    // Parse the proof and prepare for verification
    let pof = parse_groth16_proof::<Bn256>(proof_str.clone())?;

    // Verify the SNARK proof using the input hash
    let is_passed = groth16_verify(
        &pvk,
        &pof,
        &[Fr::from_str(&input_hash.to_string()).unwrap()],
    )
    .unwrap();

    // If the proof verification fails, return an error
    if !is_passed {
        return Err(ContractError::InvalidProof {
            step: String::from("AddNewKey"),
        });
    }

    // let user_balance = user_balance_of(deps.as_ref(), info.sender.as_ref())?;
    // if user_balance == Uint256::from_u128(0u128) {
    //     return Err(ContractError::Unauthorized {});
    // }

    let voice_credit_amount = VOICE_CREDIT_AMOUNT.load(deps.storage)?;

    // let voice_credit_balance = VOICECREDITBALANCE.load(deps.storage, )
    // Create a state leaf with the provided pubkey and amount
    let state_leaf = StateLeaf {
        pub_key: pubkey.clone(),
        voice_credit_balance: voice_credit_amount,
        vote_option_tree_root: Uint256::from_u128(0),
        nonce: Uint256::from_u128(0),
    }
    .hash_new_key_state_leaf(d);

    let state_index = num_sign_ups;
    // Enqueue the state leaf
    state_enqueue(&mut deps, state_leaf)?;

    num_sign_ups += Uint256::from_u128(1u128);

    NUMSIGNUPS.save(deps.storage, &num_sign_ups)?;
    SIGNUPED.save(deps.storage, pubkey.x.to_be_bytes().to_vec(), &num_sign_ups)?;

    Ok(Response::new()
        .add_attribute("action", "add_new_key")
        .add_attribute("state_idx", state_index.to_string())
        .add_attribute(
            "pubkey",
            format!("{:?},{:?}", pubkey.x.to_string(), pubkey.y.to_string()),
        )
        .add_attribute("balance", voice_credit_amount.to_string())
        .add_attribute("d0", d[0].to_string())
        .add_attribute("d1", d[1].to_string())
        .add_attribute("d2", d[2].to_string())
        .add_attribute("d3", d[3].to_string()))
}

// in voting
pub fn execute_pre_add_new_key(
    mut deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    pubkey: PubKey,
    nullifier: Uint256,
    d: [Uint256; 4],
    groth16_proof: Groth16ProofType,
) -> Result<Response, ContractError> {
    // Check if the period status is Voting
    let voting_time = VOTINGTIME.load(deps.storage)?;
    check_voting_time(env, voting_time)?;

    if NULLIFIERS.has(deps.storage, nullifier.to_be_bytes().to_vec()) {
        // Return an error response for invalid user or encrypted public key
        return Err(ContractError::NewKeyExist {});
    }

    NULLIFIERS.save(deps.storage, nullifier.to_be_bytes().to_vec(), &true)?;

    let mut num_sign_ups = NUMSIGNUPS.load(deps.storage)?;

    let max_leaves_count = MAX_LEAVES_COUNT.load(deps.storage)?;

    // // Load the scalar field value
    let snark_scalar_field =
        uint256_from_hex_string("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

    assert!(num_sign_ups < max_leaves_count, "full");
    // Check if the pubkey values are within the allowed range
    assert!(
        pubkey.x < snark_scalar_field && pubkey.y < snark_scalar_field,
        "MACI: pubkey values should be less than the snark scalar field"
    );

    let mut input: [Uint256; 7] = [Uint256::zero(); 7];

    input[0] = PRE_DEACTIVATE_ROOT.load(deps.storage)?;
    // input[1] = COORDINATORHASH.load(deps.storage)?;
    input[1] =
        uint256_from_hex_string("d53841ab0494365b341d519dcfaf0f69e375ffa406eb4484d38f55e9bdef10b");
    input[2] = nullifier;
    input[3] = d[0];
    input[4] = d[1];
    input[5] = d[2];
    input[6] = d[3];

    // Load the scalar field value
    let snark_scalar_field =
        uint256_from_hex_string("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");
    //     "21888242871839275222246405745257275088548364400416034343698204186575808495617",

    // Compute the hash of the input values
    let input_hash = uint256_from_hex_string(&hash_256_uint256_list(&input)) % snark_scalar_field; // input hash

    // Load the process verification keys
    let process_vkeys_str = GROTH16_NEWKEY_VKEYS.load(deps.storage)?;

    // Parse the SNARK proof
    let proof_str = Groth16ProofStr {
        pi_a: hex::decode(groth16_proof.a.clone())
            .map_err(|_| ContractError::HexDecodingError {})?,
        pi_b: hex::decode(groth16_proof.b.clone())
            .map_err(|_| ContractError::HexDecodingError {})?,
        pi_c: hex::decode(groth16_proof.c.clone())
            .map_err(|_| ContractError::HexDecodingError {})?,
    };

    // Parse the verification key and prepare for verification
    let vkey = parse_groth16_vkey::<Bn256>(process_vkeys_str)?;
    let pvk = prepare_verifying_key(&vkey);

    // Parse the proof and prepare for verification
    let pof = parse_groth16_proof::<Bn256>(proof_str.clone())?;

    // Verify the SNARK proof using the input hash
    let is_passed = groth16_verify(
        &pvk,
        &pof,
        &[Fr::from_str(&input_hash.to_string()).unwrap()],
    )
    .unwrap();

    // If the proof verification fails, return an error
    if !is_passed {
        return Err(ContractError::InvalidProof {
            step: String::from("PreAddNewKey"),
        });
    }

    let voice_credit_amount = VOICE_CREDIT_AMOUNT.load(deps.storage)?;

    // let voice_credit_balance = VOICECREDITBALANCE.load(deps.storage, )
    // Create a state leaf with the provided pubkey and amount
    let state_leaf = StateLeaf {
        pub_key: pubkey.clone(),
        voice_credit_balance: voice_credit_amount,
        vote_option_tree_root: Uint256::from_u128(0),
        nonce: Uint256::from_u128(0),
    }
    .hash_decativate_state_leaf();

    let state_index = num_sign_ups;
    // Enqueue the state leaf
    state_enqueue(&mut deps, state_leaf)?;

    num_sign_ups += Uint256::from_u128(1u128);

    NUMSIGNUPS.save(deps.storage, &num_sign_ups)?;
    SIGNUPED.save(deps.storage, pubkey.x.to_be_bytes().to_vec(), &num_sign_ups)?;

    Ok(Response::new()
        .add_attribute("action", "pre_add_new_key")
        .add_attribute("state_idx", state_index.to_string())
        .add_attribute(
            "pubkey",
            format!("{:?},{:?}", pubkey.x.to_string(), pubkey.y.to_string()),
        )
        .add_attribute("balance", voice_credit_amount.to_string()))
}

pub fn execute_start_process_period(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    let period = PERIOD.load(deps.storage)?;
    let voting_time = VOTINGTIME.load(deps.storage)?;

    if env.block.time <= voting_time.end_time {
        return Err(ContractError::PeriodError {});
    } else {
        if period.status == PeriodStatus::Ended
            || period.status == PeriodStatus::Processing
            || period.status == PeriodStatus::Tallying
        {
            return Err(ContractError::PeriodError {});
        }
    }

    let processed_dmsg_count = PROCESSED_DMSG_COUNT.load(deps.storage)?;
    let dmsg_chain_length = DMSG_CHAIN_LENGTH.load(deps.storage)?;

    // Check that all deactivate messages have been processed
    if processed_dmsg_count != dmsg_chain_length {
        return Err(ContractError::DmsgLeftProcess {});
    }

    // Update the period status to Processing
    let period = Period {
        status: PeriodStatus::Processing,
    };
    PERIOD.save(deps.storage, &period)?;
    // Compute the state root
    let state_root = state_root(deps.as_ref());
    // Compute the current state commitment as the hash of the state root and 0
    CURRENT_STATE_COMMITMENT.save(
        deps.storage,
        &hash2([state_root, Uint256::from_u128(0u128)]),
    )?;

    // Return a success response
    Ok(Response::new().add_attribute("action", "start_process_period"))
}

pub fn execute_process_message(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    new_state_commitment: Uint256,
    groth16_proof: Groth16ProofType,
) -> Result<Response, ContractError> {
    let period = PERIOD.load(deps.storage)?;
    // Check if the period status is Processing
    if period.status != PeriodStatus::Processing {
        return Err(ContractError::PeriodError {});
    }
    let mut processed_msg_count = PROCESSED_MSG_COUNT.load(deps.storage)?;
    let msg_chain_length = MSG_CHAIN_LENGTH.load(deps.storage)?;

    // Check that all messages have not been processed yet
    assert!(
        processed_msg_count < msg_chain_length,
        "all messages have been processed"
    );

    // Create an array to store the input values for the SNARK proof
    let mut input: [Uint256; 7] = [Uint256::zero(); 7];

    let num_sign_ups = NUMSIGNUPS.load(deps.storage)?;
    let max_vote_options = MAX_VOTE_OPTIONS.load(deps.storage)?;

    let circuit_type = CIRCUITTYPE.load(deps.storage)?;
    if circuit_type == Uint256::from_u128(0u128) {
        // 1p1v
        input[0] = (num_sign_ups << 32) + max_vote_options; // packedVals
    } else if circuit_type == Uint256::from_u128(1u128) {
        // qv
        input[0] = (num_sign_ups << 32) + (circuit_type << 64) + max_vote_options;
        // packedVals
    }

    // input[0] = (num_sign_ups << 32) + max_vote_options; // packedVals

    // Load the coordinator's public key hash
    let coordinator_hash = COORDINATORHASH.load(deps.storage)?;
    input[1] = coordinator_hash; // coordPubKeyHash

    // Load the MACI parameters
    let parameters = MACIPARAMETERS.load(deps.storage)?;
    let batch_size = parameters.message_batch_size;

    // Compute the start and end indices of the current batch
    let batch_start_index = (msg_chain_length - processed_msg_count - Uint256::from_u128(1u128))
        / batch_size
        * batch_size;
    let mut batch_end_index = batch_start_index.clone() + batch_size;
    if batch_end_index > msg_chain_length {
        batch_end_index = msg_chain_length;
    }

    // Load the hash of the message at the batch start index
    input[2] = MSG_HASHES.load(
        deps.storage,
        batch_start_index.clone().to_be_bytes().to_vec(),
    )?; // batchStartHash

    // Load the hash of the message at the batch end index
    input[3] = MSG_HASHES.load(deps.storage, batch_end_index.to_be_bytes().to_vec())?; // batchEndHash

    // Load the current state commitment
    let current_state_commitment = CURRENT_STATE_COMMITMENT.load(deps.storage)?;
    input[4] = current_state_commitment;

    // Set the new state commitment
    input[5] = new_state_commitment;
    input[6] = CURRENT_DEACTIVATE_COMMITMENT.load(deps.storage)?;

    // Load the scalar field value
    let snark_scalar_field =
        uint256_from_hex_string("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");
    //     "21888242871839275222246405745257275088548364400416034343698204186575808495617",

    // Compute the hash of the input values
    let input_hash = uint256_from_hex_string(&hash_256_uint256_list(&input)) % snark_scalar_field; // input hash

    let groth16_proof_data = groth16_proof;
    // Load the process verification keys
    let process_vkeys_str = GROTH16_PROCESS_VKEYS.load(deps.storage)?;

    // Parse the SNARK proof
    let proof_str = Groth16ProofStr {
        pi_a: hex::decode(groth16_proof_data.a.clone())
            .map_err(|_| ContractError::HexDecodingError {})?,
        pi_b: hex::decode(groth16_proof_data.b.clone())
            .map_err(|_| ContractError::HexDecodingError {})?,
        pi_c: hex::decode(groth16_proof_data.c.clone())
            .map_err(|_| ContractError::HexDecodingError {})?,
    };

    // Parse the verification key and prepare for verification
    let vkey = parse_groth16_vkey::<Bn256>(process_vkeys_str)?;
    let pvk = prepare_verifying_key(&vkey);

    // Parse the proof and prepare for verification
    let pof = parse_groth16_proof::<Bn256>(proof_str.clone())?;

    // Verify the SNARK proof using the input hash
    let is_passed = groth16_verify(
        &pvk,
        &pof,
        &[Fr::from_str(&input_hash.to_string()).unwrap()],
    )
    .unwrap();

    // If the proof verification fails, return an error
    if !is_passed {
        return Err(ContractError::InvalidProof {
            step: String::from("Process"),
        });
    }

    let attributes = vec![
        attr("zk_verify", is_passed.to_string()),
        attr("commitment", new_state_commitment.to_string()),
        attr("proof", format!("{:?}", groth16_proof_data)),
        attr("certification_system", "groth16"),
        attr("processed_msg_count", processed_msg_count.to_string()),
    ];

    // Proof verify success
    // Update the current state commitment
    CURRENT_STATE_COMMITMENT.save(deps.storage, &new_state_commitment)?;

    // Update the count of processed messages
    processed_msg_count += batch_end_index - batch_start_index;
    PROCESSED_MSG_COUNT.save(deps.storage, &processed_msg_count)?;
    Ok(Response::new()
        .add_attribute("action", "process_message")
        .add_attributes(attributes))
}

pub fn execute_stop_processing_period(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    let period = PERIOD.load(deps.storage)?;
    // Check if the period status is Processing
    if period.status != PeriodStatus::Processing {
        return Err(ContractError::PeriodError {});
    }

    let processed_msg_count = PROCESSED_MSG_COUNT.load(deps.storage)?;
    let msg_chain_length = MSG_CHAIN_LENGTH.load(deps.storage)?;

    if processed_msg_count != msg_chain_length {
        return Err(ContractError::MsgLeftProcess {});
    }

    let period = Period {
        status: PeriodStatus::Tallying,
    };
    PERIOD.save(deps.storage, &period)?;

    Ok(Response::new()
        .add_attribute("action", "stop_processing_period")
        .add_attribute("period", "Tallying"))
}

pub fn execute_process_tally(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    new_tally_commitment: Uint256,
    groth16_proof: Groth16ProofType,
) -> Result<Response, ContractError> {
    let period = PERIOD.load(deps.storage)?;
    // Check if the period status is Tallying
    if period.status != PeriodStatus::Tallying {
        return Err(ContractError::PeriodError {});
    }

    let mut processed_user_count = PROCESSED_USER_COUNT.load(deps.storage)?;
    let num_sign_ups = NUMSIGNUPS.load(deps.storage)?;
    // Check that all users have not been processed yet
    assert!(
        processed_user_count.clone() < num_sign_ups.clone(),
        "all users have been processed"
    );

    let parameters = MACIPARAMETERS.load(deps.storage)?;
    // Calculate the batch size
    let batch_size =
        Uint256::from_u128(5u128).pow(parameters.int_state_tree_depth.to_string().parse().unwrap());
    // Calculate the batch number
    let batch_num = processed_user_count / batch_size;

    // Create an array to store the input values for the SNARK proof
    let mut input: [Uint256; 4] = [Uint256::zero(); 4];

    input[0] = (num_sign_ups << 32) + batch_num; // packedVals

    // Load the current state commitment and current tally commitment
    let current_state_commitment = CURRENT_STATE_COMMITMENT.load(deps.storage)?;
    let current_tally_commitment = CURRENT_TALLY_COMMITMENT.load(deps.storage)?;

    input[1] = current_state_commitment; // stateCommitment
    input[2] = current_tally_commitment; // tallyCommitment
    input[3] = new_tally_commitment; // newTallyCommitment

    // Load the scalar field value
    let snark_scalar_field =
        uint256_from_hex_string("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");
    // let snark_scalar_field = uint256_from_decimal_string(
    //     "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    // );

    // Compute the hash of the input values
    let input_hash = uint256_from_hex_string(&hash_256_uint256_list(&input)) % snark_scalar_field;

    let groth16_proof_data = groth16_proof;
    // Load the tally verification keys
    let tally_vkeys_str = GROTH16_TALLY_VKEYS.load(deps.storage)?;

    // Parse the SNARK proof
    let proof_str = Groth16ProofStr {
        pi_a: hex::decode(groth16_proof_data.a.clone())
            .map_err(|_| ContractError::HexDecodingError {})?,
        pi_b: hex::decode(groth16_proof_data.b.clone())
            .map_err(|_| ContractError::HexDecodingError {})?,
        pi_c: hex::decode(groth16_proof_data.c.clone())
            .map_err(|_| ContractError::HexDecodingError {})?,
    };

    // Parse the verification key and prepare for verification
    let vkey = parse_groth16_vkey::<Bn256>(tally_vkeys_str)?;
    let pvk = prepare_verifying_key(&vkey);

    // Parse the proof and prepare for verification
    let pof = parse_groth16_proof::<Bn256>(proof_str.clone())?;

    // Verify the SNARK proof using the input hash
    let is_passed = groth16_verify(
        &pvk,
        &pof,
        &[Fr::from_str(&input_hash.to_string()).unwrap()],
    )
    .unwrap();

    // If the proof verification fails, return an error
    if !is_passed {
        return Err(ContractError::InvalidProof {
            step: String::from("Tally"),
        });
    }

    let attributes = vec![
        attr("zk_verify", is_passed.to_string()),
        attr("commitment", new_tally_commitment.to_string()),
        attr("proof", format!("{:?}", groth16_proof_data)),
        attr("certification_system", "groth16"),
        attr("processed_user_count", processed_user_count.to_string()),
    ];

    // Proof verify success
    // Update the current tally commitment
    CURRENT_TALLY_COMMITMENT
        .save(deps.storage, &new_tally_commitment)
        .unwrap();

    // Update the count of processed users
    processed_user_count += batch_size;

    PROCESSED_USER_COUNT
        .save(deps.storage, &processed_user_count)
        .unwrap();

    Ok(Response::new()
        .add_attribute("action", "process_tally")
        .add_attributes(attributes))
}

fn execute_stop_tallying_period(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    results: Vec<Uint256>,
    salt: Uint256,
) -> Result<Response, ContractError> {
    let period = PERIOD.load(deps.storage)?;
    // Check if the period status is Tallying
    if period.status != PeriodStatus::Tallying {
        return Err(ContractError::PeriodError {});
    }

    let tally_timeout = TALLY_TIMEOUT.load(deps.storage)?;

    let voting_time = VOTINGTIME.load(deps.storage)?;
    let current_time = env.block.time;
    let different_time = current_time.seconds() - voting_time.end_time.seconds();

    let mut attributes = vec![];
    if different_time > tally_timeout.seconds() {
        let delay_timestamp = voting_time.end_time;
        let delay_duration = different_time;
        let delay_reason = format!("Tallying has timed out after {} seconds", different_time);
        let delay_process_dmsg_count = Uint256::from_u128(0u128);
        let delay_type = DelayType::TallyDelay;

        let mut delay_records = DELAY_RECORDS.load(deps.storage)?;
        let delay_record = DelayRecord {
            delay_timestamp: delay_timestamp.clone(),
            delay_duration: delay_duration.clone(),
            delay_reason: delay_reason.clone(),
            delay_process_dmsg_count,
            delay_type,
        };
        delay_records.records.push(delay_record);
        DELAY_RECORDS.save(deps.storage, &delay_records)?;

        attributes.push(attr(
            "delay_timestamp",
            delay_timestamp.seconds().to_string(),
        ));
        attributes.push(attr("delay_duration", delay_duration.to_string()));
        attributes.push(attr("delay_reason", delay_reason));
        attributes.push(attr("delay_type", "tally_delay"));
    }

    let processed_user_count = PROCESSED_USER_COUNT.load(deps.storage)?;
    let num_sign_ups = NUMSIGNUPS.load(deps.storage)?;
    let max_vote_options = MAX_VOTE_OPTIONS.load(deps.storage)?;

    // Check that all users have been processed
    assert!(processed_user_count >= num_sign_ups);

    // Check that the number of results is not greater than the maximum vote options
    assert!(Uint256::from_u128(results.len() as u128) <= max_vote_options);

    // Load the QTR library and MACI parameters
    let qtr_lib = QTR_LIB.load(deps.storage)?;
    let parameters = MACIPARAMETERS.load(deps.storage)?;

    // Calculate the results root
    let results_root = qtr_lib.root_of(parameters.vote_option_tree_depth, results.clone());

    // Calculate the tally commitment
    let tally_commitment = hash2([results_root, salt]);

    // Load the current tally commitment
    let current_tally_commitment = CURRENT_TALLY_COMMITMENT.load(deps.storage)?;
    if current_tally_commitment == Uint256::from_u128(0u128) {
        let mut sum = Uint256::zero();

        // Save the results and calculate the sum
        for i in 0..results.len() {
            RESULT.save(
                deps.storage,
                Uint256::from_u128(i as u128).to_be_bytes().to_vec(),
                &results[i],
            )?;
            sum += results[i];
        }

        // Save the total result
        TOTAL_RESULT.save(deps.storage, &sum)?;

        // Update the period status to Ended
        let period = Period {
            status: PeriodStatus::Ended,
        };
        PERIOD.save(deps.storage, &period)?;

        return Ok(Response::new()
            .add_attribute("action", "stop_tallying_period")
            .add_attribute(
                "results",
                format!(
                    "{:?}",
                    results
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                ),
            )
            .add_attribute("all_result", sum.to_string())
            .add_attributes(attributes));
    }
    // Check that the tally commitment matches the current tally commitment
    assert_eq!(tally_commitment, current_tally_commitment);

    let mut sum = Uint256::zero();

    // Save the results and calculate the sum
    for i in 0..results.len() {
        RESULT.save(
            deps.storage,
            Uint256::from_u128(i as u128).to_be_bytes().to_vec(),
            &results[i],
        )?;
        sum += results[i];
    }

    // Save the total result
    TOTAL_RESULT.save(deps.storage, &sum)?;

    // Update the period status to Ended
    let period = Period {
        status: PeriodStatus::Ended,
    };
    PERIOD.save(deps.storage, &period)?;

    Ok(Response::new()
        .add_attribute("action", "stop_tallying_period")
        .add_attribute(
            "results",
            format!(
                "{:?}",
                results
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
            ),
        )
        .add_attribute("all_result", sum.to_string())
        .add_attributes(attributes))
}

fn execute_grant(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    max_amount: Uint128,
) -> Result<Response, ContractError> {
    // Check if the sender is authorized to execute the function
    if !is_admin(deps.as_ref(), info.sender.as_ref())? {
        return Err(ContractError::Unauthorized {});
    }

    let voting_time = VOTINGTIME.load(deps.storage)?;
    check_voting_time(env.clone(), voting_time.clone())?;

    if FEEGRANTS.exists(deps.storage) {
        return Err(ContractError::FeeGrantAlreadyExists {});
    }

    let denom = "peaka".to_string();

    let mut amount: Uint128 = Uint128::new(0);
    // Iterate through the funds and find the amount with the MACI denomination
    info.funds.iter().for_each(|fund| {
        if fund.denom == denom {
            amount = fund.amount;
        }
    });
    FEEGRANTS.save(deps.storage, &max_amount)?;

    let whitelist = WHITELIST.load(deps.storage)?;

    let base_amount = max_amount / Uint128::from(whitelist.users.len() as u128);

    let expiration_time = Some(SdkTimestamp {
        seconds: voting_time.end_time.seconds() as i64,
        nanos: 0,
    });

    let allowance = BasicAllowance {
        spend_limit: vec![SdkCoin {
            denom: denom,
            amount: base_amount.to_string(),
        }],
        expiration: expiration_time,
    };

    let allowed_allowance = AllowedMsgAllowance {
        allowance: Some(Any {
            type_url: BasicAllowance::TYPE_URL.to_string(),
            value: allowance.encode_to_vec(),
        }),
        allowed_messages: vec!["/cosmwasm.wasm.v1.MsgExecuteContract".to_string()],
    };

    let mut messages = vec![];
    for i in 0..whitelist.users.len() {
        let grant_msg = MsgGrantAllowance {
            granter: env.contract.address.to_string(),
            grantee: whitelist.users[i].addr.to_string(),
            allowance: Some(Any {
                type_url: AllowedMsgAllowance::TYPE_URL.to_string(),
                value: allowed_allowance.encode_to_vec(),
            }),
        };

        let message = CosmosMsg::Stargate {
            type_url: MsgGrantAllowance::TYPE_URL.to_string(),
            value: grant_msg.encode_to_vec().into(),
        };
        messages.push(message);
    }

    Ok(Response::default().add_messages(messages).add_attributes([
        ("action", "grant"),
        ("max_amount", max_amount.to_string().as_str()),
        ("base_amount", base_amount.to_string().as_str()),
        ("bond_amount", amount.to_string().as_str()),
    ]))
}

fn execute_revoke(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // Check if the sender is authorized to execute the function
    if !is_admin(deps.as_ref(), info.sender.as_ref())? {
        return Err(ContractError::Unauthorized {});
    }

    if !FEEGRANTS.exists(deps.storage) {
        return Err(ContractError::FeeGrantIsNotExists {});
    }

    let whitelist = WHITELIST.load(deps.storage)?;

    let mut messages = vec![];
    for i in 0..whitelist.users.len() {
        let revoke_msg = MsgRevokeAllowance {
            granter: env.contract.address.to_string(),
            grantee: whitelist.users[i].addr.to_string(),
        };
        let message = CosmosMsg::Stargate {
            type_url: MsgRevokeAllowance::TYPE_URL.to_string(),
            value: revoke_msg.encode_to_vec().into(),
        };
        messages.push(message);
    }
    FEEGRANTS.remove(deps.storage);

    Ok(Response::default()
        .add_messages(messages)
        .add_attributes([("action", "revoke")]))
}

fn execute_bond(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    if !is_admin(deps.as_ref(), info.sender.as_ref())? {
        return Err(ContractError::Unauthorized {});
    }

    let denom = "peaka".to_string();
    let mut amount: Uint128 = Uint128::new(0);
    // Iterate through the funds and find the amount with the MACI denomination
    info.funds.iter().for_each(|fund| {
        if fund.denom == denom {
            amount = fund.amount;
        }
    });

    Ok(Response::new()
        .add_attribute("action", "bond")
        .add_attribute("amount", amount.to_string()))
}

fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    if !is_admin(deps.as_ref(), info.sender.as_ref())? {
        return Err(ContractError::Unauthorized {});
    }

    let denom = "peaka".to_string();
    let contract_balance = deps.querier.query_balance(env.contract.address, &denom)?;
    let mut withdraw_amount = amount.map_or_else(|| contract_balance.amount.u128(), |am| am.u128());

    if withdraw_amount > contract_balance.amount.u128() {
        withdraw_amount = contract_balance.amount.u128();
    }

    let amount_res = coins(withdraw_amount, denom);
    let message = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: amount_res,
    };

    Ok(Response::new()
        .add_message(message)
        .add_attribute("action", "withdraw")
        .add_attribute("amount", withdraw_amount.to_string()))
}

fn can_sign_up(deps: Deps, sender: &Addr) -> StdResult<bool> {
    let cfg = WHITELIST.load(deps.storage)?;
    let is_whitelist = cfg.is_whitelist(sender);
    let is_register = cfg.is_register(sender);
    Ok(is_whitelist && !is_register)
}

// Load the root node of the state tree
fn state_root(deps: Deps) -> Uint256 {
    let root = NODES
        .load(
            deps.storage,
            Uint256::from_u128(0u128).to_be_bytes().to_vec(),
        )
        .unwrap();
    root
}

// Enqueues the state leaf into the tree
fn state_enqueue(deps: &mut DepsMut, leaf: Uint256) -> Result<bool, ContractError> {
    let leaf_idx0 = LEAF_IDX_0.load(deps.storage).unwrap();
    let num_sign_ups = NUMSIGNUPS.load(deps.storage).unwrap();

    let leaf_idx = leaf_idx0 + num_sign_ups;
    NODES.save(deps.storage, leaf_idx.to_be_bytes().to_vec(), &leaf)?;
    state_update_at(deps, leaf_idx)
}

// Updates the state at the given index in the tree
fn state_update_at(deps: &mut DepsMut, index: Uint256) -> Result<bool, ContractError> {
    let leaf_idx0 = LEAF_IDX_0.load(deps.storage).unwrap();
    if index < leaf_idx0 {
        return Err(ContractError::MustUpdate {});
    }

    let mut idx = index.clone();

    let mut height = 0;

    let zeros = ZEROS_H10.load(deps.storage).unwrap();

    while idx > Uint256::from_u128(0u128) {
        let parent_idx = (idx - Uint256::one()) / Uint256::from(5u8);
        let children_idx0 = parent_idx * Uint256::from(5u8) + Uint256::one();

        let zero = zeros[height];

        let mut inputs: [Uint256; 5] = [Uint256::zero(); 5];

        for i in 0..5 {
            let node_value = NODES
                .may_load(
                    deps.storage,
                    (children_idx0 + Uint256::from_u128(i as u128))
                        .to_be_bytes()
                        .to_vec(),
                )
                .unwrap();

            let child = match node_value {
                Some(value) => value,
                None => zero,
            };

            inputs[i] = child;
        }

        if NODES.has(deps.storage, parent_idx.to_be_bytes().to_vec()) {
            NODES
                .update(
                    deps.storage,
                    parent_idx.to_be_bytes().to_vec(),
                    |_c: Option<Uint256>| -> StdResult<_> { Ok(hash5(inputs)) },
                )
                .unwrap();
        } else {
            NODES
                .save(
                    deps.storage,
                    parent_idx.to_be_bytes().to_vec(),
                    &hash5(inputs),
                )
                .unwrap();
        }

        height += 1;
        idx = parent_idx;
    }

    Ok(true)
}

fn check_voting_time(env: Env, voting_time: VotingTime) -> Result<(), ContractError> {
    let current_time = env.block.time;

    // Check if the current time is within the voting time range (inclusive of start and end time)
    if current_time < voting_time.start_time || current_time > voting_time.end_time {
        return Err(ContractError::PeriodError {});
    }

    Ok(())
}

pub fn hash_message_and_enc_pub_key(
    message: MessageData,
    enc_pub_key: PubKey,
    prev_hash: Uint256,
) -> Uint256 {
    let mut m: [Uint256; 5] = [Uint256::zero(); 5];
    m[0] = message.data[0];
    m[1] = message.data[1];
    m[2] = message.data[2];
    m[3] = message.data[3];
    m[4] = message.data[4];

    let mut n: [Uint256; 5] = [Uint256::zero(); 5];
    n[0] = message.data[5];
    n[1] = message.data[6];
    n[2] = enc_pub_key.x;
    n[3] = enc_pub_key.y;
    n[4] = prev_hash;

    let m_hash = hash5(m);

    let n_hash = hash5(n);
    let m_n_hash = hash2([m_hash, n_hash]);
    return m_n_hash;
}

// Only admin can execute
fn is_admin(deps: Deps, sender: &str) -> StdResult<bool> {
    let cfg = ADMIN.load(deps.storage)?;
    let can = cfg.is_admin(&sender);
    Ok(can)
}

// Only operator can execute
fn is_operator(deps: Deps, sender: &str) -> StdResult<bool> {
    let operator = MACI_OPERATOR.load(deps.storage)?;
    let can_operator = sender.to_string() == operator.to_string();
    Ok(can_operator)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin {} => to_json_binary(&ADMIN.load(deps.storage)?.admin),
        QueryMsg::Operator {} => to_json_binary(&MACI_OPERATOR.load(deps.storage)?),
        QueryMsg::GetRoundInfo {} => {
            to_json_binary::<RoundInfo>(&ROUNDINFO.load(deps.storage).unwrap())
        }
        QueryMsg::GetVotingTime {} => {
            to_json_binary::<VotingTime>(&VOTINGTIME.load(deps.storage).unwrap())
        }
        QueryMsg::GetPeriod {} => to_json_binary::<Period>(&PERIOD.load(deps.storage).unwrap()),
        QueryMsg::GetNumSignUp {} => {
            to_json_binary::<Uint256>(&NUMSIGNUPS.may_load(deps.storage)?.unwrap_or_default())
        }
        QueryMsg::GetMsgChainLength {} => {
            to_json_binary::<Uint256>(&MSG_CHAIN_LENGTH.may_load(deps.storage)?.unwrap_or_default())
        }
        QueryMsg::GetDMsgChainLength {} => to_json_binary::<Uint256>(
            &DMSG_CHAIN_LENGTH
                .may_load(deps.storage)?
                .unwrap_or_default(),
        ),
        QueryMsg::GetProcessedDMsgCount {} => to_json_binary::<Uint256>(
            &PROCESSED_DMSG_COUNT
                .may_load(deps.storage)?
                .unwrap_or_default(),
        ),
        QueryMsg::GetProcessedMsgCount {} => to_json_binary::<Uint256>(
            &PROCESSED_MSG_COUNT
                .may_load(deps.storage)?
                .unwrap_or_default(),
        ),
        QueryMsg::GetProcessedUserCount {} => to_json_binary::<Uint256>(
            &PROCESSED_USER_COUNT
                .may_load(deps.storage)?
                .unwrap_or_default(),
        ),
        QueryMsg::GetResult { index } => to_json_binary::<Uint256>(
            &RESULT
                .may_load(deps.storage, index.to_be_bytes().to_vec())?
                .unwrap_or_default(),
        ),
        QueryMsg::GetAllResult {} => {
            to_json_binary::<Uint256>(&TOTAL_RESULT.may_load(deps.storage)?.unwrap_or_default())
        }
        QueryMsg::GetStateIdxInc { address } => to_json_binary::<Uint256>(
            &STATEIDXINC
                .may_load(deps.storage, &address)?
                .unwrap_or_default(),
        ),
        QueryMsg::GetVoiceCreditBalance { index } => to_json_binary::<Uint256>(
            &VOICECREDITBALANCE
                .load(deps.storage, index.to_be_bytes().to_vec())
                .unwrap(),
        ),
        QueryMsg::GetVoiceCreditAmount {} => to_json_binary::<Uint256>(
            &VOICE_CREDIT_AMOUNT
                .may_load(deps.storage)?
                .unwrap_or_default(),
        ),
        QueryMsg::WhiteList {} => to_json_binary::<Whitelist>(&query_white_list(deps)?),
        QueryMsg::CanSignUp { sender } => {
            to_json_binary::<bool>(&query_can_sign_up(deps, &sender)?)
        }
        QueryMsg::IsWhiteList { sender } => to_json_binary::<bool>(&is_whitelist(deps, &sender)?),
        QueryMsg::IsRegister { sender } => to_json_binary::<bool>(&is_register(deps, &sender)?),
        QueryMsg::Signuped { pubkey_x } => to_json_binary::<Uint256>(
            &SIGNUPED
                .load(deps.storage, pubkey_x.to_be_bytes().to_vec())
                .unwrap(),
        ),
        QueryMsg::VoteOptionMap {} => {
            to_json_binary::<Vec<String>>(&VOTEOPTIONMAP.load(deps.storage).unwrap())
        }
        QueryMsg::MaxVoteOptions {} => {
            to_json_binary::<Uint256>(&MAX_VOTE_OPTIONS.may_load(deps.storage)?.unwrap_or_default())
        }
        QueryMsg::QueryTotalFeeGrant {} => {
            to_json_binary::<Uint128>(&FEEGRANTS.may_load(deps.storage)?.unwrap_or_default())
        }
        QueryMsg::QueryCircuitType {} => {
            to_json_binary::<Uint256>(&CIRCUITTYPE.may_load(deps.storage)?.unwrap_or_default())
        }
        QueryMsg::QueryCertSystem {} => {
            to_json_binary::<Uint256>(&CERTSYSTEM.may_load(deps.storage)?.unwrap_or_default())
        }
        QueryMsg::QueryPreDeactivateRoot {} => to_json_binary::<Uint256>(
            &PRE_DEACTIVATE_ROOT
                .may_load(deps.storage)?
                .unwrap_or_default(),
        ),
        QueryMsg::GetDelayRecords {} => {
            let records = DELAY_RECORDS
                .may_load(deps.storage)?
                .unwrap_or(DelayRecords { records: vec![] });
            to_json_binary(&records)
        }
    }
}

pub fn query_white_list(deps: Deps) -> StdResult<Whitelist> {
    let cfg = WHITELIST.load(deps.storage)?;
    Ok(Whitelist {
        users: cfg.users.into_iter().map(|a| a.into()).collect(),
    })
}

pub fn query_can_sign_up(deps: Deps, sender: &Addr) -> StdResult<bool> {
    Ok(can_sign_up(deps, &sender)?)
}

pub fn is_whitelist(deps: Deps, sender: &Addr) -> StdResult<bool> {
    let cfg = WHITELIST.load(deps.storage)?;
    let is_whitelist = cfg.is_whitelist(sender);
    Ok(is_whitelist)
}

pub fn is_register(deps: Deps, sender: &Addr) -> StdResult<bool> {
    let cfg = WHITELIST.load(deps.storage)?;
    let is_register = cfg.is_register(sender);
    Ok(is_register)
}

// pub fn query_user_balance_of(deps: Deps, sender: String) -> StdResult<Uint256> {
//     Ok(user_balance_of(deps, &sender)?)
// }

#[cfg(test)]
mod tests {}

// Check if the operator has processed all deactivate messages within 15 minutes
pub fn check_operator_process_time(deps: Deps, env: Env) -> Result<bool, ContractError> {
    let current_time = env.block.time;

    let first_dmsg_time = match FIRST_DMSG_TIMESTAMP.may_load(deps.storage)? {
        Some(timestamp) => timestamp,
        None => return Ok(true), // 如果没有第一条消息的时间戳,说明还没有deactivate消息需要处理
    };

    let processed_dmsg_count = PROCESSED_DMSG_COUNT.load(deps.storage)?;
    let dmsg_chain_length = DMSG_CHAIN_LENGTH.load(deps.storage)?;

    // 如果当前批次已经处理完,返回true
    if processed_dmsg_count == dmsg_chain_length {
        return Ok(true);
    }

    let time_difference = current_time.seconds() - first_dmsg_time.seconds();

    let deactivate_timeout = DEACTIVATE_TIMEOUT.load(deps.storage)?;
    if time_difference > deactivate_timeout.seconds() {
        return Ok(false);
    }

    Ok(true)
}

// Check if tally is completed within 6 hours after the voting end time
fn check_stop_tallying_time(deps: Deps, env: Env) -> Result<bool, ContractError> {
    let voting_time = VOTINGTIME.load(deps.storage)?;
    let current_time = env.block.time;

    // If the current time is less than or equal to the voting end time, it means we're still in the voting period.
    if current_time <= voting_time.end_time {
        return Ok(true);
    }

    let period = PERIOD.load(deps.storage)?;

    // If the period is already Ended, it means tally has been successfully completed
    if period.status == PeriodStatus::Ended {
        return Ok(true);
    }
    let tally_timeout = TALLY_TIMEOUT.load(deps.storage)?;

    // Check if we're within the tally window after the voting end time
    let time_difference = current_time.seconds() - voting_time.end_time.seconds();
    if time_difference > tally_timeout.seconds() {
        return Ok(false);
    }

    Ok(true)
}

#[cw_serde]
pub struct OperatorPerformance {
    pub unprocessed_deactivate_count: Uint256,
    pub delay_process_count: Uint256,
    pub deactivate_processing_complete: bool,
    pub tally_processing_complete: bool,
    pub period: PeriodStatus,
    pub miss_rate: Uint256, // Changed from Decimal to Uint256
}

pub fn calculate_operator_performance(
    deps: Deps,
    env: Env,
) -> Result<OperatorPerformance, ContractError> {
    let penalty_rate = PENALTY_RATE.load(deps.storage)?;

    let voting_time = VOTINGTIME.load(deps.storage)?;
    let current_time = env.block.time;
    let processed_dmsg_count = PROCESSED_DMSG_COUNT.load(deps.storage)?;
    let dmsg_chain_length = DMSG_CHAIN_LENGTH.load(deps.storage)?;

    // Check if the divisor is zero
    if dmsg_chain_length.is_zero() {
        return Err(ContractError::DivisionByZero {});
    }
    let delay_records = DELAY_RECORDS.load(deps.storage)?;
    let delay_process_count = Uint256::from_u128(delay_records.records.len() as u128);
    let unprocessed_deactivate_count = dmsg_chain_length - processed_dmsg_count;

    // Calculate base miss rate as percentage (0-100)
    let base_miss_rate = if dmsg_chain_length.is_zero() {
        Uint256::zero()
    } else {
        unprocessed_deactivate_count.multiply_ratio(Uint256::from(100u128), dmsg_chain_length)
    };

    let (period, miss_rate) = if current_time < voting_time.start_time {
        (PeriodStatus::Pending, Uint256::zero())
    } else if current_time <= voting_time.end_time {
        (PeriodStatus::Voting, base_miss_rate)
    } else {
        let period_state = PERIOD.load(deps.storage)?;
        let final_miss_rate = if check_stop_tallying_time(deps, env.clone())? {
            base_miss_rate
        } else {
            // If base_miss_rate is less than PENALTY_RATE, set it to 0 (maximum penalty)
            // Otherwise, increase the penalty by PENALTY_RATE
            if base_miss_rate <= penalty_rate {
                Uint256::zero()
            } else {
                base_miss_rate - penalty_rate
            }
        };
        (period_state.status, final_miss_rate)
    };

    Ok(OperatorPerformance {
        unprocessed_deactivate_count,
        delay_process_count,
        deactivate_processing_complete: check_operator_process_time(deps, env.clone())?,
        tally_processing_complete: check_stop_tallying_time(deps, env)?,
        period,
        miss_rate,
    })
}
