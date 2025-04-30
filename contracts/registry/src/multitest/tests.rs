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

pub fn next_block_6_minutes(block: &mut BlockInfo) {
    block.time = block.time.plus_minutes(6);
    block.height += 1;
}

pub fn next_block_11_minutes(block: &mut BlockInfo) {
    block.time = block.time.plus_minutes(11);
    block.height += 1;
}

pub fn next_block_22_minutes(block: &mut BlockInfo) {
    block.time = block.time.plus_minutes(22);
    block.height += 1;
}

pub fn next_block_31_minutes(block: &mut BlockInfo) {
    block.time = block.time.plus_minutes(31);
    block.height += 1;
}   

pub fn next_block_3_hours(block: &mut BlockInfo) {
    block.time = block.time.plus_hours(3);
    block.height += 1;
}


pub fn next_block_4_days(block: &mut BlockInfo) {
    block.time = block.time.plus_days(4);
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

    let small_base_payamount = 20000000000000000000u128; // 20 DORA
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

    let small_base_payamount = 20000000000000000000u128; // 20 DORA

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
                    a: "04c5d564a7dd1feaba7c422f429327bd5e9430cb6b67f0bf77a19788fac264a7080063a86a7f45a4893f68ce20a4ee0bc22cb085866c9387d1b822d1b1fba033".to_string(),
                    b: "1515ff2d529baece55d6d9f7338de646dc83fba060dce13a88a8b31114b9df8b2573959072de506962aeadc60198138bfbba84a7ed3a7a349563a1b3ed4fef67062efab826e3b0ebdbce3bf0744634ba3db1d336d7ba38cfd16b8d3d42f9bb5d2546e2f71e1bbd6f680e65696aad163f99c3baac18c27146c17086542b2da535".to_string(),
                    c: "2cb72b2822ff424c48e6972bdca59ee9f6b813bfb00571a286c41070a5a56de91d5e9c1310eef0653dc5c34255ebd40afaffcd65ba34f6d4799a4dca92cf12ff".to_string()
                };
                println!("process_deactivate_message proof {:?}", proof);
                println!(
                    "process_deactivate_message new state commitment {:?}",
                    new_deactivate_commitment
                );
                app.update_block(next_block_11_minutes);
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
                                a: "2d72823f8f7e44117ab51f945a9b14899b56f8bc24ac93b68a42a0d3df1b815109535ff45d1694ad8299b55499065d5cd1fc03a222439d8f58ab0a3369d06739".to_string(),
                                b: "2a83d66d8de353f284d8ab3d4d6beac737a8fd4528df53019f4f480bf224755525c910f7f36d59b8d5254cb79d20a229e0e0522c1b1d5501dbec4b71f8929bc6136cdc1cd58d2dfd73d52c7c6387a459420a7f2aba924a57b3eb32fc4d7d5dc311a262196bb79b95a66b5ebd6ccb89b0a06fb400d031f0ea0fe6c340b709cd69".to_string(),
                                c: "184fc89744396e6069a589c662b41502cc7adc69feace7e07bb3a56ca24e100b087f4604dde84b86d75d581d4d5278369776076ad940f08f2d6d44651a599b8d".to_string()
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
                app.update_block(next_block_11_minutes);

                let sign_up_after_voting_end_error = maci_contract
                    .amaci_sign_up(
                        &mut app,
                        Addr::unchecked(3.to_string()),
                        test_pubkey.clone(),
                    )
                    .unwrap_err();
                assert_eq!(
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
                    a: "1d15cc37dcb272bbacd6793212775a2660be0df34138806c176b23b237b342811de490c5864a44aaff70e9f85d622b3f9a96f4a75b8f95ccabbb653b9e03bfdf".to_string(),
                    b: "0de712129a430f3172d4ccd0241a95babbe777145ff386a56e6637ae968ae10b01f7c7d3edf83e66dfa8b022f1a8297e6f4c188950d82eb72565cc1bfaf28511185f0579f703687c4410e6b21cbb30efc82cd4841b56ccf9fa66341a0c6c256b1820517f1eb3b06e7736cdd3b170b23bec6968f02e8ed6e4fab098b42652361e".to_string(),
                    c: "097f71ae3a88062baabafc37e8d4356d00b4577c0fca127235922654ee91cd040d61065aa98b440a4a16829eb0def9ac7d03a1ab5f3b0c6219ff19bebd122b79".to_string()
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
                    a: "1e0bd768d465ded5a9e699f052dd636be8ab48fe0c4a1d3d9cacf8d36913d8ae177af12ba0069288a86d9203373c5f8f60c37cbb3398b63df1b11991e6ca0c50".to_string(),
                    b: "23dfa545c123e67c4c9fef8390afe9a3484df2058fa00d92ffc8f1f3379dcc1800584aff26615a81ab5f28d35e596b7b88d6cfd3b3ec205a8981f91aacd8aa6b2896a841947bd200cce364e04064c0b16533015c68c09d8dc2a4c6fa6e907c5b1fc331016aa03f8059a2bf73aa39b69ad5c36f07a9aede6817144e84dbfd9250".to_string(),
                    c: "2bc0d5643a8fdc18712e4e0815f698539c3ed905fa8edd091a752d679270967109ac6f774c8b8bb002bfb07675cdeb4e3bf4ec2ade2bf6e5ed74c167dc3dee0a".to_string()
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
                    AmaciContractError::PeriodError {},
                    withdraw_error.downcast().unwrap()
                );

                app.update_block(next_block_3_hours);
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
                    delay_duration: 660,
                    delay_reason:
                        "Processing of 2 deactivate messages has timed out after 660 seconds"
                            .to_string(),
                    delay_process_dmsg_count: Uint256::from_u128(2),
                    delay_type: DelayType::DeactivateDelay,
                },
                DelayRecord {
                    delay_timestamp: Timestamp::from_nanos(1571798684879000000),
                    delay_duration: 10860,
                    delay_reason: "Tallying has timed out after 10860 seconds (total process: 6, allowed: 3600 seconds)".to_string(),
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

    // app.update_block(next_block_4_days); // after 4 days, operator reward is 0, all funds are returned to admin
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
    // let operator_reward = Uint128::from(0u128); // after 4 days, operator reward is 0, all funds are returned to admin
    let penalty_amount = claim_amount - operator_reward;
    println!("operator_reward: {:?}", operator_reward);
    println!("penalty_amount: {:?}", penalty_amount);
    assert_eq!(
        operator_balance.amount,
        operator_reward + operator_balance_before_claim.amount
        // Uint128::from(0u128) // after 4 days, operator reward is 0, all funds are returned to admin
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

    let tally_delay = maci_contract.amaci_query_tally_delay(&app).unwrap();
    println!("tally_delay: {:?}", tally_delay);
}


#[test]
fn create_round_with_voting_time_qv_amaci_after_4_days_with_no_operator_reward_should_works() {
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

    let small_base_payamount = 20000000000000000000u128; // 20 DORA

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
                    a: "04c5d564a7dd1feaba7c422f429327bd5e9430cb6b67f0bf77a19788fac264a7080063a86a7f45a4893f68ce20a4ee0bc22cb085866c9387d1b822d1b1fba033".to_string(),
                    b: "1515ff2d529baece55d6d9f7338de646dc83fba060dce13a88a8b31114b9df8b2573959072de506962aeadc60198138bfbba84a7ed3a7a349563a1b3ed4fef67062efab826e3b0ebdbce3bf0744634ba3db1d336d7ba38cfd16b8d3d42f9bb5d2546e2f71e1bbd6f680e65696aad163f99c3baac18c27146c17086542b2da535".to_string(),
                    c: "2cb72b2822ff424c48e6972bdca59ee9f6b813bfb00571a286c41070a5a56de91d5e9c1310eef0653dc5c34255ebd40afaffcd65ba34f6d4799a4dca92cf12ff".to_string()
                };
                println!("process_deactivate_message proof {:?}", proof);
                println!(
                    "process_deactivate_message new state commitment {:?}",
                    new_deactivate_commitment
                );
                app.update_block(next_block_11_minutes);
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
                                a: "2d72823f8f7e44117ab51f945a9b14899b56f8bc24ac93b68a42a0d3df1b815109535ff45d1694ad8299b55499065d5cd1fc03a222439d8f58ab0a3369d06739".to_string(),
                                b: "2a83d66d8de353f284d8ab3d4d6beac737a8fd4528df53019f4f480bf224755525c910f7f36d59b8d5254cb79d20a229e0e0522c1b1d5501dbec4b71f8929bc6136cdc1cd58d2dfd73d52c7c6387a459420a7f2aba924a57b3eb32fc4d7d5dc311a262196bb79b95a66b5ebd6ccb89b0a06fb400d031f0ea0fe6c340b709cd69".to_string(),
                                c: "184fc89744396e6069a589c662b41502cc7adc69feace7e07bb3a56ca24e100b087f4604dde84b86d75d581d4d5278369776076ad940f08f2d6d44651a599b8d".to_string()
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
                app.update_block(next_block_11_minutes);

                let sign_up_after_voting_end_error = maci_contract
                    .amaci_sign_up(
                        &mut app,
                        Addr::unchecked(3.to_string()),
                        test_pubkey.clone(),
                    )
                    .unwrap_err();
                assert_eq!(
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
                    a: "1d15cc37dcb272bbacd6793212775a2660be0df34138806c176b23b237b342811de490c5864a44aaff70e9f85d622b3f9a96f4a75b8f95ccabbb653b9e03bfdf".to_string(),
                    b: "0de712129a430f3172d4ccd0241a95babbe777145ff386a56e6637ae968ae10b01f7c7d3edf83e66dfa8b022f1a8297e6f4c188950d82eb72565cc1bfaf28511185f0579f703687c4410e6b21cbb30efc82cd4841b56ccf9fa66341a0c6c256b1820517f1eb3b06e7736cdd3b170b23bec6968f02e8ed6e4fab098b42652361e".to_string(),
                    c: "097f71ae3a88062baabafc37e8d4356d00b4577c0fca127235922654ee91cd040d61065aa98b440a4a16829eb0def9ac7d03a1ab5f3b0c6219ff19bebd122b79".to_string()
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
                    a: "1e0bd768d465ded5a9e699f052dd636be8ab48fe0c4a1d3d9cacf8d36913d8ae177af12ba0069288a86d9203373c5f8f60c37cbb3398b63df1b11991e6ca0c50".to_string(),
                    b: "23dfa545c123e67c4c9fef8390afe9a3484df2058fa00d92ffc8f1f3379dcc1800584aff26615a81ab5f28d35e596b7b88d6cfd3b3ec205a8981f91aacd8aa6b2896a841947bd200cce364e04064c0b16533015c68c09d8dc2a4c6fa6e907c5b1fc331016aa03f8059a2bf73aa39b69ad5c36f07a9aede6817144e84dbfd9250".to_string(),
                    c: "2bc0d5643a8fdc18712e4e0815f698539c3ed905fa8edd091a752d679270967109ac6f774c8b8bb002bfb07675cdeb4e3bf4ec2ade2bf6e5ed74c167dc3dee0a".to_string()
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
                    AmaciContractError::PeriodError {},
                    withdraw_error.downcast().unwrap()
                );

                app.update_block(next_block_3_hours);
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
                    delay_duration: 660,
                    delay_reason:
                        "Processing of 2 deactivate messages has timed out after 660 seconds"
                            .to_string(),
                    delay_process_dmsg_count: Uint256::from_u128(2),
                    delay_type: DelayType::DeactivateDelay,
                },
                DelayRecord {
                    delay_timestamp: Timestamp::from_nanos(1571798684879000000),
                    delay_duration: 10860,
                    delay_reason: "Tallying has timed out after 10860 seconds (total process: 6, allowed: 3600 seconds)".to_string(),
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

    app.update_block(next_block_4_days); // after 4 days, operator reward is 0, all funds are returned to admin
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
    let operator_reward = claim_amount.multiply_ratio(100u128 - (50u128 + 5u128 * 1), 100u128);
    let operator_reward = Uint128::from(0u128); // after 4 days, operator reward is 0, all funds are returned to admin
    let penalty_amount = claim_amount - operator_reward;
    println!("operator_reward: {:?}", operator_reward);
    println!("penalty_amount: {:?}", penalty_amount);
    assert_eq!(
        operator_balance.amount,
        // operator_reward + operator_balance_before_claim.amount
        Uint128::from(0u128) // after 4 days, operator reward is 0, all funds are returned to admin
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

    let tally_delay = maci_contract.amaci_query_tally_delay(&app).unwrap();
    println!("tally_delay: {:?}", tally_delay);
}
