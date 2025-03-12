use cosmwasm_std::{coins, from_json, Addr, BlockInfo, Timestamp, Uint128, Uint256};
use cw_multi_test::App;

// use crate::error::ContractError;
// use crate::msg::ClaimsResponse;
use crate::{
    multitest::{
        operator, operator2, operator3, operator_pubkey1, operator_pubkey2, operator_pubkey3,
        owner, user1, user2, user3, user4, AmaciRegistryCodeId, InstantiationData, DORA_DEMON,
    },
    state::ValidatorSet,
};
use cw_amaci::multitest::{MaciCodeId, MaciContract};
use cw_amaci::ContractError as AmaciContractError;

use cw_amaci::msg::Groth16ProofType;
use cw_amaci::multitest::uint256_from_decimal_string;
use cw_amaci::state::{
    DelayRecord, DelayRecords, DelayType, MessageData, Period, PeriodStatus, PubKey,
};
use cw_multi_test::next_block;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MsgData {
    input_hash: String,
    packed_vals: String,
    batch_start_hash: String,
    batch_end_hash: String,
    msgs: Vec<Vec<String>>,
    coord_priv_key: String,
    coord_pub_key: Vec<String>,
    enc_pub_keys: Vec<Vec<String>>,
    current_state_root: String,
    current_state_leaves: Vec<Vec<String>>,
    current_state_leaves_path_elements: Vec<Vec<Vec<String>>>,
    current_state_commitment: String,
    current_state_salt: String,
    new_state_commitment: String,
    new_state_salt: String,
    current_vote_weights: Vec<String>,
    current_vote_weights_path_elements: Vec<Vec<Vec<String>>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TallyData {
    state_root: String,
    state_salt: String,
    packed_vals: String,
    state_commitment: String,
    current_tally_commitment: String,
    new_tally_commitment: String,
    input_hash: String,
    state_leaf: Vec<Vec<String>>,
    state_path_elements: Vec<Vec<String>>,
    votes: Vec<Vec<String>>,
    current_results: Vec<String>,
    current_results_root_salt: String,
    new_results_root_salt: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResultData {
    results: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserPubkeyData {
    pubkeys: Vec<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AMaciLogEntry {
    #[serde(rename = "type")]
    log_type: String,
    data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetStateLeafData {
    leaf_idx: String,
    pub_key: Vec<String>,
    balance: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PublishDeactivateMessageData {
    message: Vec<String>,
    enc_pub_key: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProofDeactivateData {
    size: String,
    new_deactivate_commitment: String,
    new_deactivate_root: String,
    proof: Groth16Proof,
}

#[derive(Debug, Serialize, Deserialize)]
struct Groth16Proof {
    pi_a: Vec<String>,
    pi_b: Vec<Vec<String>>,
    pi_c: Vec<String>,
    protocol: String,
    curve: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProofAddNewKeyData {
    pub_key: Vec<String>,
    proof: Groth16Proof,
    d: Vec<String>,
    nullifier: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PublishMessageData {
    message: Vec<String>,
    enc_pub_key: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProcessMessageData {
    proof: Groth16Proof,
    new_state_commitment: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProcessTallyData {
    proof: Groth16Proof,
    new_tally_commitment: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StopTallyingPeriodData {
    results: Vec<String>,
    salt: String,
}

fn deserialize_data<T: serde::de::DeserializeOwned>(data: &serde_json::Value) -> T {
    serde_json::from_value(data.clone()).expect("Unable to deserialize data")
}

pub fn next_block_6_seconds(block: &mut BlockInfo) {
    block.time = block.time.plus_seconds(6);
    block.height += 1;
}

pub fn next_block_11_min(block: &mut BlockInfo) {
    block.time = block.time.plus_minutes(11);
    block.height += 1;
}

pub fn next_block_3_days(block: &mut BlockInfo) {
    block.time = block.time.plus_days(3);
    block.height += 1;
}

// // #[test]
// fn instantiate_should_works() {
//     let user1_coin_amount = 30u128;
//     let user2_coin_amount = 20u128;
//     let user3_coin_amount = 10u128;

//     let mut app = App::new(|router, _api, storage| {
//         router
//             .bank
//             .init_balance(storage, &user1(), coins(user1_coin_amount, DORA_DEMON))
//             .unwrap();
//         router
//             .bank
//             .init_balance(storage, &user2(), coins(user2_coin_amount, DORA_DEMON))
//             .unwrap();

//         router
//             .bank
//             .init_balance(storage, &user3(), coins(user3_coin_amount, DORA_DEMON))
//             .unwrap();
//         // router
//         //     .bank
//         //     .init_balance(storage, &(), coins(500, ARCH_DEMON))
//         //     .unwrap();
//     });

//     let code_id = AmaciRegistryCodeId::store_code(&mut app);
//     let label = "Dora AMaci Registry";
//     let contract = code_id.instantiate(&mut app, owner(), 1u64, label).unwrap();

//     let not_admin_or_operator_set_validators =
//         contract.set_validators(&mut app, user1()).unwrap_err();
//     assert_eq!(
//         ContractError::Unauthorized {},
//         not_admin_or_operator_set_validators.downcast().unwrap()
//     );

//     _ = contract.set_validators(&mut app, owner());

//     let validator_set = contract.get_validators(&app).unwrap();
//     assert_eq!(
//         ValidatorSet {
//             addresses: vec![user1(), user2(), user4()]
//         },
//         validator_set
//     );

//     _ = contract.set_maci_operator(&mut app, user1(), operator());
//     let user1_operator_addr = contract.get_validator_operator(&app, user1()).unwrap();
//     assert_eq!(operator(), user1_operator_addr);

//     _ = contract.set_maci_operator_pubkey(&mut app, operator(), operator_pubkey1());
//     let user1_operator_pubkey = contract.get_operator_pubkey(&app, operator()).unwrap();
//     assert_eq!(pubkey1(), user1_operator_pubkey);

//     _ = contract.remove_validator(&mut app, owner(), user4());
//     let validator_set_after_remove_user4 = contract.get_validators(&app).unwrap();
//     assert_eq!(
//         ValidatorSet {
//             addresses: vec![user1(), user2()]
//         },
//         validator_set_after_remove_user4
//     );

//     let not_validator_set_operator_error = contract
//         .set_maci_operator(&mut app, user3(), operator())
//         .unwrap_err();
//     assert_eq!(
//         ContractError::Unauthorized {},
//         not_validator_set_operator_error.downcast().unwrap()
//     );

//     _ = contract.set_maci_operator(&mut app, user2(), operator2());
//     let user2_operator_addr = contract.get_validator_operator(&app, user2()).unwrap();
//     assert_eq!(operator2(), user2_operator_addr);
//     _ = contract.set_maci_operator_pubkey(&mut app, operator2(), pubkey2());
//     let user2_operator_pubkey = contract.get_operator_pubkey(&app, operator2()).unwrap();
//     assert_eq!(pubkey2(), user2_operator_pubkey);
//     _ = contract.set_maci_operator_pubkey(&mut app, operator2(), pubkey3());
//     let user2_operator_pubkey3 = contract.get_operator_pubkey(&app, operator2()).unwrap();
//     assert_eq!(pubkey3(), user2_operator_pubkey3);

//     _ = contract.set_validators_all(&mut app, owner());
//     _ = contract.remove_validator(&mut app, owner(), user2());
//     let validator_set_after_remove_user2 = contract.get_validators(&app).unwrap();
//     assert_eq!(
//         ValidatorSet {
//             addresses: vec![user1(), user3()]
//         },
//         validator_set_after_remove_user2
//     );

//     let removed_validator_cannot_set_operator = contract
//         .set_maci_operator(&mut app, user2(), operator3())
//         .unwrap_err();
//     assert_eq!(
//         ContractError::Unauthorized {},
//         removed_validator_cannot_set_operator.downcast().unwrap()
//     );

//     let cannot_set_same_operator_address = contract
//         .set_maci_operator(&mut app, user3(), operator())
//         .unwrap_err();
//     assert_eq!(
//         ContractError::ExistedMaciOperator {},
//         cannot_set_same_operator_address.downcast().unwrap()
//     );

//     _ = contract.set_maci_operator(&mut app, user3(), operator3());
//     let user3_operator_addr = contract.get_validator_operator(&app, user3()).unwrap();
//     assert_eq!(operator3(), user3_operator_addr);

//     let user3_register_with_user1_pubkey = contract
//         .set_maci_operator_pubkey(&mut app, operator3(), pubkey1())
//         .unwrap_err();
//     assert_eq!(
//         ContractError::PubkeyExisted {},
//         user3_register_with_user1_pubkey.downcast().unwrap()
//     );

//     _ = contract.set_maci_operator_pubkey(&mut app, operator3(), pubkey3());
//     let user3_operator_pubkey = contract.get_operator_pubkey(&app, operator3()).unwrap();
//     assert_eq!(pubkey3(), user3_operator_pubkey);
// }

// // #[test]
// fn create_round_should_works() {
//     let user1_coin_amount = 30u128;

//     let mut app = App::new(|router, _api, storage| {
//         router
//             .bank
//             .init_balance(storage, &user1(), coins(user1_coin_amount, DORA_DEMON))
//             .unwrap();
//     });

//     let register_code_id = AmaciRegistryCodeId::store_code(&mut app);
//     let amaci_code_id = MaciCodeId::store_default_code(&mut app);

//     let label = "Dora AMaci Registry";
//     let contract = register_code_id
//         .instantiate(&mut app, owner(), amaci_code_id.id(), label)
//         .unwrap();

//     _ = contract.set_validators(&mut app, owner());

//     let validator_set = contract.get_validators(&app).unwrap();
//     assert_eq!(
//         ValidatorSet {
//             addresses: vec![user1(), user2(), user4()]
//         },
//         validator_set
//     );

//     _ = contract.set_maci_operator(&mut app, user1(), operator());
//     let user1_operator_addr = contract.get_validator_operator(&app, user1()).unwrap();
//     assert_eq!(operator(), user1_operator_addr);

//     let user1_check_operator = contract.is_maci_operator(&app, operator()).unwrap();

//     assert_eq!(true, user1_check_operator);

//     _ = contract.set_maci_operator_pubkey(&mut app, operator(), pubkey1());

//     let user1_operator_pubkey = contract.get_operator_pubkey(&app, operator()).unwrap();
//     assert_eq!(pubkey1(), user1_operator_pubkey);

//     let create_round_with_wrong_circuit_type = contract
//         .create_round(
//             &mut app,
//             user1(),
//             operator(),
//             Uint256::from_u128(2u128),
//             Uint256::from_u128(0u128),
//         )
//         .unwrap_err();
//     assert_eq!(
//         AmaciContractError::UnsupportedCircuitType {},
//         create_round_with_wrong_circuit_type.downcast().unwrap()
//     );

//     let create_round_with_wrong_certification_system = contract
//         .create_round(
//             &mut app,
//             user1(),
//             operator(),
//             Uint256::from_u128(0u128),
//             Uint256::from_u128(1u128),
//         )
//         .unwrap_err();
//     assert_eq!(
//         AmaciContractError::UnsupportedCertificationSystem {},
//         create_round_with_wrong_certification_system
//             .downcast()
//             .unwrap()
//     );

//     let resp = contract
//         .create_round(
//             &mut app,
//             user1(),
//             operator(),
//             Uint256::from_u128(0u128),
//             Uint256::from_u128(0u128),
//         )
//         .unwrap();

//     let amaci_contract_addr: InstantiationData = from_json(&resp.data.unwrap()).unwrap();
//     println!("{:?}", amaci_contract_addr);
//     let maci_contract = MaciContract::new(amaci_contract_addr.addr);
//     let amaci_admin = maci_contract.query_admin(&app).unwrap();
//     println!("{:?}", amaci_admin);
//     assert_eq!(user1(), amaci_admin);

//     let amaci_operator = maci_contract.query_operator(&app).unwrap();
//     println!("{:?}", amaci_operator);
//     assert_eq!(operator(), amaci_operator);

//     let amaci_round_info = maci_contract.query_round_info(&app).unwrap();
//     println!("{:?}", amaci_round_info);
// }

#[test]
fn create_round_with_reward_should_works() {
    let owner_coin_amount = 1000000000000000000000u128; // 1000 DORA (register 500, create round 50)

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &owner(), coins(owner_coin_amount, DORA_DEMON))
            .unwrap();
    });

    let register_code_id = AmaciRegistryCodeId::store_code(&mut app);
    let amaci_code_id = MaciCodeId::store_default_code(&mut app);

    let label = "Dora AMaci Registry";
    let contract = register_code_id
        .instantiate(&mut app, owner(), amaci_code_id.id(), label)
        .unwrap();

    _ = contract.set_validators(&mut app, owner());

    let validator_set = contract.get_validators(&app).unwrap();
    assert_eq!(
        ValidatorSet {
            addresses: vec![user1(), user2(), user4()]
        },
        validator_set
    );

    _ = contract.set_maci_operator(&mut app, user1(), operator());
    let user1_operator_addr = contract.get_validator_operator(&app, user1()).unwrap();
    assert_eq!(operator(), user1_operator_addr);

    let user1_check_operator = contract.is_maci_operator(&app, operator()).unwrap();

    assert_eq!(true, user1_check_operator);

    _ = contract.set_maci_operator_pubkey(&mut app, operator(), operator_pubkey1());

    let user1_operator_pubkey = contract.get_operator_pubkey(&app, operator()).unwrap();
    assert_eq!(operator_pubkey1(), user1_operator_pubkey);

    // _ = contract.migrate_v1(&mut app, owner(), amaci_code_id.id()).unwrap();

    let small_base_payamount = 50000000000000000000u128;
    let create_round_with_wrong_circuit_type = contract
        .create_round(
            &mut app,
            owner(),
            operator(),
            Uint256::from_u128(2u128),
            Uint256::from_u128(0u128),
            &coins(small_base_payamount, DORA_DEMON),
        )
        .unwrap_err();
    assert_eq!(
        AmaciContractError::UnsupportedCircuitType {},
        create_round_with_wrong_circuit_type.downcast().unwrap()
    );

    let create_round_with_wrong_certification_system = contract
        .create_round(
            &mut app,
            owner(),
            operator(),
            Uint256::from_u128(0u128),
            Uint256::from_u128(1u128),
            &coins(small_base_payamount, DORA_DEMON),
        )
        .unwrap_err();
    assert_eq!(
        AmaciContractError::UnsupportedCertificationSystem {},
        create_round_with_wrong_certification_system
            .downcast()
            .unwrap()
    );

    let resp = contract
        .create_round(
            &mut app,
            owner(),
            operator(),
            Uint256::from_u128(0u128),
            Uint256::from_u128(0u128),
            &coins(small_base_payamount, DORA_DEMON),
        )
        .unwrap();

    let amaci_contract_addr: InstantiationData = from_json(&resp.data.unwrap()).unwrap();
    println!("{:?}", amaci_contract_addr);
    let maci_contract = MaciContract::new(amaci_contract_addr.addr.clone());
    let amaci_admin = maci_contract.amaci_query_admin(&app).unwrap();
    println!("{:?}", amaci_admin);
    assert_eq!(owner(), amaci_admin);

    let amaci_operator = maci_contract.amaci_query_operator(&app).unwrap();
    println!("{:?}", amaci_operator);
    assert_eq!(operator(), amaci_operator);

    let amaci_round_info = maci_contract.amaci_query_round_info(&app).unwrap();
    println!("{:?}", amaci_round_info);

    let amaci_round_balance = contract
        .balance_of(
            &app,
            amaci_contract_addr.addr.to_string(),
            DORA_DEMON.to_string(),
        )
        .unwrap();
    let circuit_charge_config = contract.get_circuit_charge_config(&app).unwrap();
    let total_fee = Uint128::from(small_base_payamount);
    let admin_fee = circuit_charge_config.fee_rate * total_fee;
    let operator_fee = total_fee - admin_fee;

    assert_eq!(Uint128::from(operator_fee), amaci_round_balance.amount);
}

#[test]
fn create_round_with_voting_time_qv_amaci_should_works() {
    let msg_file_path = "./src/test/qv_test/msg.json";

    let mut msg_file = fs::File::open(msg_file_path).expect("Failed to open file");
    let mut msg_content = String::new();

    msg_file
        .read_to_string(&mut msg_content)
        .expect("Failed to read file");

    let data: MsgData = serde_json::from_str(&msg_content).expect("Failed to parse JSON");

    let pubkey_file_path = "./src/test/user_pubkey.json";

    let mut pubkey_file = fs::File::open(pubkey_file_path).expect("Failed to open file");
    let mut pubkey_content = String::new();

    pubkey_file
        .read_to_string(&mut pubkey_content)
        .expect("Failed to read file");
    let pubkey_data: UserPubkeyData =
        serde_json::from_str(&pubkey_content).expect("Failed to parse JSON");

    let logs_file_path = "./src/test/amaci_test/logs.json";

    let mut logs_file = fs::File::open(logs_file_path).expect("Failed to open file");
    let mut logs_content = String::new();

    logs_file
        .read_to_string(&mut logs_content)
        .expect("Failed to read file");

    let logs_data: Vec<AMaciLogEntry> =
        serde_json::from_str(&logs_content).expect("Failed to parse JSON");

    let owner_coin_amount = 50000000000000000000u128; // 50 DORA (register 500, create round 50)
    let _operator_coin_amount = 1000000000000000000000u128; // 1000 DORA

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &owner(), coins(owner_coin_amount, DORA_DEMON))
            .unwrap();
        // router
        //     .bank
        //     .init_balance(
        //         storage,
        //         &operator(),
        //         coins(operator_coin_amount, DORA_DEMON),
        //     )
        //     .unwrap();
    });

    let register_code_id = AmaciRegistryCodeId::store_code(&mut app);
    let amaci_code_id = MaciCodeId::store_default_code(&mut app);

    let label = "Dora AMaci Registry";
    let contract = register_code_id
        .instantiate(&mut app, owner(), amaci_code_id.id(), label)
        .unwrap();

    _ = contract.set_validators(&mut app, owner());

    let validator_set = contract.get_validators(&app).unwrap();
    assert_eq!(
        ValidatorSet {
            addresses: vec![user1(), user2(), user4()]
        },
        validator_set
    );

    _ = contract.set_maci_operator(&mut app, user1(), operator());
    let user1_operator_addr = contract.get_validator_operator(&app, user1()).unwrap();
    assert_eq!(operator(), user1_operator_addr);

    let user1_check_operator = contract.is_maci_operator(&app, operator()).unwrap();

    assert_eq!(true, user1_check_operator);

    _ = contract.set_maci_operator_pubkey(&mut app, operator(), operator_pubkey1());

    let user1_operator_pubkey = contract.get_operator_pubkey(&app, operator()).unwrap();
    assert_eq!(operator_pubkey1(), user1_operator_pubkey);

    // _ = contract.migrate_v1(&mut app, owner(), amaci_code_id.id()).unwrap();

    let small_base_payamount = 50000000000000000000u128;

    let resp = contract
        .create_round_with_whitelist(
            &mut app,
            owner(),
            operator(),
            Uint256::from_u128(1u128),
            Uint256::from_u128(0u128),
            &coins(small_base_payamount, DORA_DEMON),
        )
        .unwrap();

    let amaci_contract_addr: InstantiationData = from_json(&resp.data.unwrap()).unwrap();
    println!("{:?}", amaci_contract_addr);
    let maci_contract = MaciContract::new(amaci_contract_addr.addr.clone());
    let amaci_admin = maci_contract.amaci_query_admin(&app).unwrap();
    println!("{:?}", amaci_admin);
    assert_eq!(owner(), amaci_admin);

    let amaci_operator = maci_contract.amaci_query_operator(&app).unwrap();
    println!("{:?}", amaci_operator);
    assert_eq!(operator(), amaci_operator);

    let amaci_round_info = maci_contract.amaci_query_round_info(&app).unwrap();
    println!("{:?}", amaci_round_info);

    let amaci_round_balance = contract
        .balance_of(
            &app,
            amaci_contract_addr.addr.to_string(),
            DORA_DEMON.to_string(),
        )
        .unwrap();
    let circuit_charge_config = contract.get_circuit_charge_config(&app).unwrap();
    let total_fee = Uint128::from(small_base_payamount);
    let admin_fee = circuit_charge_config.fee_rate * total_fee;
    let operator_fee = total_fee - admin_fee;

    assert_eq!(Uint128::from(operator_fee), amaci_round_balance.amount);

    // let start_voting_error = contract.start_voting(&mut app, owner()).unwrap_err();

    // assert_eq!(
    //     ContractError::AlreadySetVotingTime {
    //         time_name: String::from("start_time")
    //     },
    //     start_voting_error.downcast().unwrap()
    // );

    let num_sign_up = maci_contract.amaci_num_sign_up(&app).unwrap();
    assert_eq!(num_sign_up, Uint256::from_u128(0u128));

    let vote_option_map = maci_contract.amaci_vote_option_map(&app).unwrap();
    let max_vote_options = maci_contract.amaci_max_vote_options(&app).unwrap();
    assert_eq!(vote_option_map, vec!["", "", "", "", ""]);
    assert_eq!(max_vote_options, Uint256::from_u128(5u128));
    _ = maci_contract.amaci_set_vote_option_map(&mut app, owner());
    let new_vote_option_map = maci_contract.amaci_vote_option_map(&app).unwrap();
    assert_eq!(
        new_vote_option_map,
        vec![
            String::from("did_not_vote"),
            String::from("yes"),
            String::from("no"),
            String::from("no_with_veto"),
            String::from("abstain"),
        ]
    );
    // assert_eq!(num_sign_up, Uint256::from_u128(0u128));

    let test_pubkey = PubKey {
        x: uint256_from_decimal_string(&data.current_state_leaves[0][0]),
        y: uint256_from_decimal_string(&data.current_state_leaves[0][1]),
    };
    let sign_up_error = maci_contract
        .amaci_sign_up(
            &mut app,
            Addr::unchecked(0.to_string()),
            test_pubkey.clone(),
        )
        .unwrap_err();
    assert_eq!(
        AmaciContractError::PeriodError {},
        sign_up_error.downcast().unwrap()
    ); // 不能在voting环节之前进行signup

    _ = maci_contract.amaci_set_vote_option_map(&mut app, owner());

    app.update_block(next_block); // Start Voting
    let set_whitelist_only_in_pending = maci_contract
        .amaci_set_whitelist(&mut app, owner())
        .unwrap_err();
    assert_eq!(
        // 注册之后不能再进行注册
        AmaciContractError::PeriodError {},
        set_whitelist_only_in_pending.downcast().unwrap()
    );
    let set_vote_option_map_error = maci_contract
        .amaci_set_vote_option_map(&mut app, owner())
        .unwrap_err();
    assert_eq!(
        AmaciContractError::PeriodError {},
        set_vote_option_map_error.downcast().unwrap()
    );

    let error_start_process_in_voting = maci_contract
        .amaci_start_process(&mut app, owner())
        .unwrap_err();
    assert_eq!(
        AmaciContractError::PeriodError {},
        error_start_process_in_voting.downcast().unwrap()
    );
    assert_eq!(
        Period {
            status: PeriodStatus::Pending
        },
        maci_contract.amaci_get_period(&app).unwrap()
    );

    let pubkey0 = PubKey {
        x: uint256_from_decimal_string(&pubkey_data.pubkeys[0][0]),
        y: uint256_from_decimal_string(&pubkey_data.pubkeys[0][1]),
    };

    let pubkey1 = PubKey {
        x: uint256_from_decimal_string(&pubkey_data.pubkeys[1][0]),
        y: uint256_from_decimal_string(&pubkey_data.pubkeys[1][1]),
    };

    let _ = maci_contract.amaci_sign_up(&mut app, Addr::unchecked("0"), pubkey0.clone());

    let can_sign_up_error = maci_contract
        .amaci_sign_up(&mut app, Addr::unchecked("0"), pubkey0.clone())
        .unwrap_err();
    assert_eq!(
        AmaciContractError::UserAlreadyRegistered {},
        can_sign_up_error.downcast().unwrap()
    );

    let _ = maci_contract.amaci_sign_up(&mut app, Addr::unchecked("1"), pubkey1.clone());

    assert_eq!(
        maci_contract.amaci_num_sign_up(&app).unwrap(),
        Uint256::from_u128(2u128)
    );

    assert_eq!(
        maci_contract.amaci_signuped(&app, pubkey0.x).unwrap(),
        Uint256::from_u128(1u128)
    );
    assert_eq!(
        maci_contract.amaci_signuped(&app, pubkey1.x).unwrap(),
        Uint256::from_u128(2u128)
    );

    for entry in &logs_data {
        match entry.log_type.as_str() {
            "publishDeactivateMessage" => {
                let data: PublishDeactivateMessageData = deserialize_data(&entry.data);

                let message = MessageData {
                    data: [
                        uint256_from_decimal_string(&data.message[0]),
                        uint256_from_decimal_string(&data.message[1]),
                        uint256_from_decimal_string(&data.message[2]),
                        uint256_from_decimal_string(&data.message[3]),
                        uint256_from_decimal_string(&data.message[4]),
                        uint256_from_decimal_string(&data.message[5]),
                        uint256_from_decimal_string(&data.message[6]),
                    ],
                };

                let enc_pub = PubKey {
                    x: uint256_from_decimal_string(&data.enc_pub_key[0]),
                    y: uint256_from_decimal_string(&data.enc_pub_key[1]),
                };
                _ = maci_contract.amaci_publish_deactivate_message(
                    &mut app,
                    user2(),
                    message,
                    enc_pub,
                );
            }
            "proofDeactivate" => {
                let data: ProofDeactivateData = deserialize_data(&entry.data);
                assert_eq!(
                    maci_contract.amaci_dmsg_length(&app).unwrap(),
                    Uint256::from_u128(2u128)
                );

                let size = uint256_from_decimal_string(&data.size);
                let new_deactivate_commitment =
                    uint256_from_decimal_string(&data.new_deactivate_commitment);
                let new_deactivate_root = uint256_from_decimal_string(&data.new_deactivate_root);
                let proof = Groth16ProofType {
                                a: "2fac29af2cad382c07952b42c10b282d6ee5c27032548c370fdf40c693965b98239bb54fb0546480075f7e93f7f46acdacfecf3eb40fb3c16f9b13287d15fd7a".to_string(),
                                b: "18fb4503928bda6fc6aa377170b80fb3e2c73161c78c936bca222cb233318c7517ca194640de6b7790ec65ea7e46891089567d86a9fe8e419ad5e5d27e2cf96a2cf5383ef516ea8d14754c2e9e132fe566dd32eb23cd0de3543398a03a1c15f02a75014c4db8598d472112b292bbdde2968c409b759dbe76dec21da24b09d1a1".to_string(),
                                c: "18f024873175339f2e939c8bc8a369daa56257564f3e23b0cf4b635e5721f0d1285e5d66fc1dd69f581a2b146083267e4ce9a3c21e46f488af2ed9289bd00714".to_string()
                            };
                println!("process_deactivate_message proof {:?}", proof);
                println!(
                    "process_deactivate_message new state commitment {:?}",
                    new_deactivate_commitment
                );
                app.update_block(next_block_6_seconds);
                _ = maci_contract
                    .amaci_process_deactivate_message(
                        &mut app,
                        owner(),
                        size,
                        new_deactivate_commitment,
                        new_deactivate_root,
                        proof,
                    )
                    .unwrap();
            }
            "proofAddNewKey" => {
                let data: ProofAddNewKeyData = deserialize_data(&entry.data);

                let new_key_pub = PubKey {
                    x: uint256_from_decimal_string(&data.pub_key[0]),
                    y: uint256_from_decimal_string(&data.pub_key[1]),
                };

                let d: [Uint256; 4] = [
                    uint256_from_decimal_string(&data.d[0]),
                    uint256_from_decimal_string(&data.d[1]),
                    uint256_from_decimal_string(&data.d[2]),
                    uint256_from_decimal_string(&data.d[3]),
                ];

                let nullifier = uint256_from_decimal_string(&data.nullifier);

                let proof = Groth16ProofType {
                                a: "29eb173553d340b41108fa7581371d1e2eb84962e93e667aff45ee2cc05aa9b91234d82ac4caafd2eaf597e1da25c5982bef8b0a937a7f68b84954f042d4ed0f".to_string(),
                                b: "01a6d17acb0c2381082e1c35baee57af4bf393dbd94377bac54bfec15916c0b80197c2a0c0faa491e9b32b32de526c03b2c57a126eeafcb72feae194b3f8a60f0a81e4f7aa16ba2afb45a694dcc5832531b36c060f3ae31a8df0e7c724961e130d5fc5a83a7d658b63611dd37e0790b3602072529743cf727a371f82c3c250b2".to_string(),
                                c: "2e18f57e4618cac5b0111a6ca470a193dfbad5f393a455b06be2b2dbd8bb7b8e1c0f4fbb35a51d466d665d7fcfb22ea3717c6503e45f104167c4639fd01a1285".to_string()
                            };

                println!("add_new_key proof {:?}", proof);
                _ = maci_contract
                    .amaci_add_key(&mut app, owner(), new_key_pub, nullifier, d, proof)
                    .unwrap();
            }
            "publishMessage" => {
                let data: PublishMessageData = deserialize_data(&entry.data);

                let message = MessageData {
                    data: [
                        uint256_from_decimal_string(&data.message[0]),
                        uint256_from_decimal_string(&data.message[1]),
                        uint256_from_decimal_string(&data.message[2]),
                        uint256_from_decimal_string(&data.message[3]),
                        uint256_from_decimal_string(&data.message[4]),
                        uint256_from_decimal_string(&data.message[5]),
                        uint256_from_decimal_string(&data.message[6]),
                    ],
                };

                let enc_pub = PubKey {
                    x: uint256_from_decimal_string(&data.enc_pub_key[0]),
                    y: uint256_from_decimal_string(&data.enc_pub_key[1]),
                };

                println!("------- publishMessage ------");
                _ = maci_contract.amaci_publish_message(&mut app, user2(), message, enc_pub);
            }
            "processMessage" => {
                let data: ProcessMessageData = deserialize_data(&entry.data);
                app.update_block(next_block_11_min);

                let sign_up_after_voting_end_error = maci_contract
                    .amaci_sign_up(
                        &mut app,
                        Addr::unchecked(3.to_string()),
                        test_pubkey.clone(),
                    )
                    .unwrap_err();
                assert_eq!(
                    // 不能投票环节结束之后不能进行sign up
                    AmaciContractError::PeriodError {},
                    sign_up_after_voting_end_error.downcast().unwrap()
                );

                // let stop_voting_error = contract.stop_voting(&mut app, owner()).unwrap_err();
                // assert_eq!(
                //     ContractError::AlreadySetVotingTime {
                //         time_name: String::from("end_time")
                //     },
                //     stop_voting_error.downcast().unwrap()
                // );
                app.update_block(next_block);

                _ = maci_contract.amaci_start_process(&mut app, owner());
                assert_eq!(
                    Period {
                        status: PeriodStatus::Processing
                    },
                    maci_contract.amaci_get_period(&app).unwrap()
                );

                println!(
                    "after start process: {:?}",
                    maci_contract.amaci_get_period(&app).unwrap()
                );

                let error_stop_processing_with_not_finish_process = maci_contract
                    .amaci_stop_processing(&mut app, owner())
                    .unwrap_err();
                assert_eq!(
                    AmaciContractError::MsgLeftProcess {},
                    error_stop_processing_with_not_finish_process
                        .downcast()
                        .unwrap()
                );

                let new_state_commitment = uint256_from_decimal_string(&data.new_state_commitment);
                let proof = Groth16ProofType {
                        a: "096d1b959a9d1a4414da11a08220f034cd897b9cdd4cfce9a5427dca6302da15256a4aa11311f51231d905883f2224bdda75e8d1757f701164d11a286dd3831a".to_string(),
                        b: "1bce75878f05c102f04f23d8033ecbf074420c02d977e312f6e121b1254bfe262e465cd3e76211e75251ee9a1dadfaedbde55c55af20c3b906ee4f360f079f6b069f7aa8774a1fbfcca147a2581d64885a977aa05f5a03b0f8871abc2f0611ad058a4f480bdff6ec3616be5b4f197f8a96c21b206485827f628b1f8ee5f55be4".to_string(),
                        c: "11d9887f25fb3cac4a4cc9f20031f9be14bb206fe5db157ae2302d23c65ed6ad06af73ef894c43bbd89008b720efded006a0499bc76cdfc5ae8d8bf67360f37e".to_string()
                    };
                println!("process_message proof {:?}", proof);
                println!(
                    "process_message new state commitment {:?}",
                    new_state_commitment
                );
                println!("------ processMessage ------");
                _ = maci_contract
                    .amaci_process_message(&mut app, owner(), new_state_commitment, proof)
                    .unwrap();
            }
            "processTally" => {
                let data: ProcessTallyData = deserialize_data(&entry.data);

                _ = maci_contract.amaci_stop_processing(&mut app, owner());
                println!(
                    "after stop process: {:?}",
                    maci_contract.amaci_get_period(&app).unwrap()
                );

                let error_start_process_in_talling = maci_contract
                    .amaci_start_process(&mut app, owner())
                    .unwrap_err();
                assert_eq!(
                    AmaciContractError::PeriodError {},
                    error_start_process_in_talling.downcast().unwrap()
                );
                assert_eq!(
                    Period {
                        status: PeriodStatus::Tallying
                    },
                    maci_contract.amaci_get_period(&app).unwrap()
                );

                let new_tally_commitment = uint256_from_decimal_string(&data.new_tally_commitment);

                let tally_proof = Groth16ProofType {
                            a: "2a88fe840fa2eb49979acee8b545766fc83f28f128219041d3bf1e900fcf86a219b124cbf9b755a802186c391a154137eadf865a4b604452f5f98c4c533a1652".to_string(),
                            b: "0b6706904e637f888a9406db1529c84c26d068ad54bbfd3597de3e542f9230302cfdfdcd606e3544a63139b888fa561c2bf5ed928826d68c4f35e0fd07d491da27488896f67e261e8e3c6e33f947700b10eb6029daf6d9ae19add49e19fde2792563eec2f3fa6a43b1ec42c7d2f32b644c2f18e2b48d5dc552958b49c30f80c8".to_string(),
                            c: "0d4529ea7dd9c686c5673d48ee6a3fb3971a7cf12441f40e0ba6116046d64767288254a628cac0e46ccd3f1d1100ba2cd922d71066ae48b78283773b505665e0".to_string()
                        };

                _ = maci_contract
                    .amaci_process_tally(&mut app, owner(), new_tally_commitment, tally_proof)
                    .unwrap();
            }
            "stopTallyingPeriod" => {
                let data: StopTallyingPeriodData = deserialize_data(&entry.data);

                let results: Vec<Uint256> = vec![
                    uint256_from_decimal_string(&data.results[0]),
                    uint256_from_decimal_string(&data.results[1]),
                    uint256_from_decimal_string(&data.results[2]),
                    uint256_from_decimal_string(&data.results[3]),
                    uint256_from_decimal_string(&data.results[4]),
                ];

                let salt = uint256_from_decimal_string(&data.salt);

                let withdraw_error = maci_contract.amaci_claim(&mut app, owner()).unwrap_err();
                assert_eq!(
                    AmaciContractError::ClaimMustAfterThirdDay {},
                    withdraw_error.downcast().unwrap()
                );

                app.update_block(next_block_11_min);
                _ = maci_contract.amaci_stop_tallying(&mut app, owner(), results, salt);

                let all_result = maci_contract.amaci_get_all_result(&app);
                println!("all_result: {:?}", all_result);
                let error_start_process = maci_contract
                    .amaci_start_process(&mut app, owner())
                    .unwrap_err();
                assert_eq!(
                    AmaciContractError::PeriodError {},
                    error_start_process.downcast().unwrap()
                );

                assert_eq!(
                    Period {
                        status: PeriodStatus::Ended
                    },
                    maci_contract.amaci_get_period(&app).unwrap()
                );
            }
            _ => println!("Unknown type: {}", entry.log_type),
        }
    }

    let delay_records = maci_contract.amaci_query_delay_records(&app).unwrap();
    println!("delay_records: {:?}", delay_records);
    assert_eq!(
        delay_records,
        DelayRecords {
            records: vec![
                DelayRecord {
                    delay_timestamp: Timestamp::from_nanos(1571797424879305533),
                    delay_duration: 6,
                    delay_reason:
                        "Processing of 2 deactivate messages has timed out after 6 seconds"
                            .to_string(),
                    delay_process_dmsg_count: Uint256::from_u128(2),
                    delay_type: DelayType::DeactivateDelay,
                },
                DelayRecord {
                    delay_timestamp: Timestamp::from_nanos(1571798084879000000),
                    delay_duration: 671,
                    delay_reason: "Tallying has timed out after 671 seconds".to_string(),
                    delay_process_dmsg_count: Uint256::from_u128(0),
                    delay_type: DelayType::TallyDelay,
                },
            ]
        }
    );

    let round_balance_before_claim = contract
        .balance_of(
            &app,
            maci_contract.addr().to_string(),
            DORA_DEMON.to_string(),
        )
        .unwrap();
    println!(
        "round_balance_before_claim: {:?}",
        round_balance_before_claim
    );
    // assert_eq!(round_balance_before_claim.amount, operator_fee.amount);

    let owner_balance_before_claim = contract
        .balance_of(&app, owner().to_string(), DORA_DEMON.to_string())
        .unwrap();
    println!(
        "owner_balance_before_claim: {:?}",
        owner_balance_before_claim
    );
    // assert_eq!(owner_balance.amount, round_balance_before_withdraw.amount + Uint128::from(delay));

    let operator_balance_before_claim = contract
        .balance_of(&app, operator().to_string(), DORA_DEMON.to_string())
        .unwrap();
    println!(
        "operator_balance_before_claim: {:?}",
        operator_balance_before_claim
    );
    // assert_eq!(operator_balance.amount, Uint128::from(0u128));

    app.update_block(next_block_3_days);
    _ = maci_contract.amaci_claim(&mut app, owner());
    let owner_balance = contract
        .balance_of(&app, owner().to_string(), DORA_DEMON.to_string())
        .unwrap();
    println!("owner_balance: {:?}", owner_balance);
    // assert_eq!(owner_balance.amount, round_balance_before_withdraw.amount + Uint128::from(delay));

    let operator_balance = contract
        .balance_of(&app, operator().to_string(), DORA_DEMON.to_string())
        .unwrap();
    println!("operator_balance: {:?}", operator_balance);
    // assert_eq!(operator_balance.amount, Uint128::from(0u128));

    let round_balance_after_claim = contract
        .balance_of(
            &app,
            maci_contract.addr().to_string(),
            DORA_DEMON.to_string(),
        )
        .unwrap();
    println!(
        "round_balance_after_claim: {:?}",
        round_balance_after_claim
    );
    // assert_eq!(round_balance_after_claim.amount, Uint128::from(0u128));

    let claim_amount = Uint128::from(round_balance_before_claim.amount);
    let operator_reward = claim_amount.multiply_ratio(100u128 - (50u128 + 5u128 * 2), 100u128);
    let penalty_amount = claim_amount - operator_reward;
    println!("operator_reward: {:?}", operator_reward);
    println!("penalty_amount: {:?}", penalty_amount);
    assert_eq!(
        operator_balance.amount,
        operator_reward + operator_balance_before_claim.amount
    );
    assert_eq!(
        owner_balance.amount,
        penalty_amount + owner_balance_before_claim.amount
    );
    assert_eq!(
        round_balance_after_claim.amount,
        round_balance_before_claim.amount - claim_amount
    );
    assert_eq!(round_balance_after_claim.amount, Uint128::from(0u128));

    let claim_error = maci_contract.amaci_claim(&mut app, owner()).unwrap_err();
    assert_eq!(
        AmaciContractError::AllFundsClaimed {},
        claim_error.downcast().unwrap()
    );
}
