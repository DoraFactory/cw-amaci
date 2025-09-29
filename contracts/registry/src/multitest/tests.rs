use cosmwasm_std::{
    coins, from_json, Addr, BalanceResponse, BankMsg, BlockInfo, CosmosMsg, Empty, Timestamp,
    Uint128, Uint256,
};
use cw_multi_test::{AppResponse, CosmosRouter, Executor, FailingModule, Stargate};
use crate::error::ContractError as RegistryContractError;

// use crate::error::ContractError;
// use crate::msg::ClaimsResponse;
use crate::{
    multitest::{
        admin, creator, operator, operator_pubkey1, user1, user2, user4, AmaciRegistryCodeId, InstantiationData,
        DORA_DEMON,
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
use prost::Message;
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
    pi_a: String,
    pi_b: String,
    pi_c: String,
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

// Helper: use AMACI's DefaultApp (StargateAccepting) for compatibility
fn create_dora_app() -> cw_amaci::multitest::App {
    cw_amaci::multitest::create_app()
}

fn find_attr(resp: &cw_multi_test::AppResponse, key: &str) -> Option<String> {
    for ev in &resp.events {
        for attr in &ev.attributes {
            if attr.key == key {
                return Some(attr.value.clone());
            }
        }
    }
    None
}

// ========== Sponsor Stargate + DoraApp for sponsor withdraw flow tests ==========
// Mock stargate to simulate sponsor module withdraw to recipient by transferring
// all balance from derived sponsor address to recipient
struct SponsorStargate;

impl Stargate for SponsorStargate {
    fn execute<ExecC, QueryC>(
        &self,
        api: &dyn cosmwasm_std::Api,
        storage: &mut dyn cosmwasm_std::Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &cosmwasm_std::BlockInfo,
        _sender: Addr,
        type_url: String,
        value: cosmwasm_std::Binary,
    ) -> cw_multi_test::error::AnyResult<AppResponse>
    where
        ExecC: std::fmt::Debug + Clone + PartialEq + schemars::JsonSchema + serde::de::DeserializeOwned + 'static,
        QueryC: cosmwasm_std::CustomQuery + serde::de::DeserializeOwned + 'static,
    {
        if type_url == "/doravota.sponsor.v1.MsgWithdrawSponsorFunds" {
            // decode message
            let msg = crate::msg::MsgWithdrawSponsorFunds::decode(value.as_slice()).unwrap();
            let round_addr = Addr::unchecked(msg.contract_address);
            // Use the same fallback derivation as contract in test env
            let sponsor_addr = Addr::unchecked(format!("{}_sponsor", round_addr.as_str()));

            // query sponsor balance
            let bal_bin = router.query(
                api,
                storage,
                block,
                cosmwasm_std::QueryRequest::Bank(cosmwasm_std::BankQuery::Balance {
                    address: sponsor_addr.to_string(),
                    denom: DORA_DEMON.to_string(),
                }),
            )?;
            let coin: BalanceResponse = cosmwasm_std::from_json(bal_bin).unwrap();
            let amount = coin.amount.amount;
            if amount.is_zero() {
                return Ok(AppResponse::default());
            }
            let send = CosmosMsg::<ExecC>::Bank(BankMsg::Send {
                to_address: msg.recipient,
                amount: coins(amount.u128(), DORA_DEMON),
            });
            // execute as sponsor address (fund owner)
            let resp = router.execute(api, storage, block, sponsor_addr, send)?;
            return Ok(resp);
        }
        Ok(AppResponse::default())
    }
}

type DoraApp = cw_multi_test::App<
    cw_multi_test::BankKeeper,
    cosmwasm_std::testing::MockApi,
    cosmwasm_std::testing::MockStorage,
    FailingModule<Empty, Empty, Empty>,
    cw_multi_test::WasmKeeper<Empty, Empty>,
    cw_multi_test::StakeKeeper,
    cw_multi_test::DistributionKeeper,
    cw_multi_test::IbcFailingModule,
    cw_multi_test::GovFailingModule,
    SponsorStargate,
>;

fn create_app_dora() -> DoraApp {
    cw_multi_test::AppBuilder::new()
        .with_stargate(SponsorStargate)
        .build(cw_multi_test::no_init)
}

fn instantiate_registry_dora(app: &mut DoraApp, amaci_code_id: u64) -> Addr {
    let code = cw_multi_test::ContractWrapper::new(super::super::contract::execute, super::super::contract::instantiate, super::super::contract::query)
        .with_reply(super::super::contract::reply);
    let code_id = app.store_code(Box::new(code));
    let msg = crate::msg::InstantiateMsg {
        admin: admin(),
        operator: operator(),
        amaci_code_id,
    };
    app.instantiate_contract(code_id, creator(), &msg, &[], "Dora AMaci Registry", Some(creator().to_string()))
        .unwrap()
}

fn setup_operator_dora(app: &mut DoraApp, registry: &Addr) {
    // set validators
    let _ = app
        .execute_contract(
            admin(),
            registry.clone(),
            &crate::msg::ExecuteMsg::SetValidators {
                addresses: super::super::state::ValidatorSet {
                    addresses: vec![user1(), user2(), user4()],
                },
            },
            &[],
        )
        .unwrap();
    // bind operator for user1
    let _ = app
        .execute_contract(
            user1(),
            registry.clone(),
            &crate::msg::ExecuteMsg::SetMaciOperator { operator: operator() },
            &[],
        )
        .unwrap();
    // set operator pubkey
    let _ = app
        .execute_contract(
            operator(),
            registry.clone(),
            &crate::msg::ExecuteMsg::SetMaciOperatorPubkey {
                pubkey: operator_pubkey1(),
            },
            &[],
        )
        .unwrap();
}

// new tests moved to a dedicated file to avoid type conflicts

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
    let admin_coin_amount = 1000000000000000000000u128; // 1000 DORA (register 500, create round 50)
    let creator_coin_amount = 1000000000000000000000u128; // 1000 DORA

    let mut app = cw_amaci::multitest::create_app();
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &admin(), coins(admin_coin_amount, DORA_DEMON))
            .unwrap();
        router
            .bank
            .init_balance(storage, &creator(), coins(creator_coin_amount, DORA_DEMON))
            .unwrap();
    });

    let register_code_id = AmaciRegistryCodeId::store_code(&mut app);
    let amaci_code_id = MaciCodeId::store_default_code(&mut app);

    let label = "Dora AMaci Registry";
    let contract = register_code_id
        .instantiate(&mut app, creator(), amaci_code_id.id(), label)
        .unwrap();

    _ = contract.set_validators(&mut app, admin());

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

    let small_base_payamount = 120000000000000000000u128; // total small fee: 20 + 100 DORA
    let create_round_with_wrong_circuit_type = contract
        .create_round(
            &mut app,
            creator(),
            operator(),
            Uint256::from_u128(2u128),
            Uint256::from_u128(0u128),
            &coins(120000000000000000000u128, DORA_DEMON),
        )
        .unwrap_err();
    assert_eq!(
        AmaciContractError::UnsupportedCircuitType {},
        create_round_with_wrong_circuit_type.downcast().unwrap()
    );

    let create_round_with_wrong_certification_system = contract
        .create_round(
            &mut app,
            creator(),
            operator(),
            Uint256::from_u128(0u128),
            Uint256::from_u128(1u128),
            &coins(120000000000000000000u128, DORA_DEMON),
        )
        .unwrap_err();
    assert_eq!(
        AmaciContractError::UnsupportedCertificationSystem {},
        create_round_with_wrong_certification_system
            .downcast()
            .unwrap()
    );

    // Record creator balance before creating round
    let creator_balance_before = contract
        .balance_of(&app, creator().to_string(), DORA_DEMON.to_string())
        .unwrap();

    // Record admin balance before creating round
    let admin_balance_before = contract
        .balance_of(&app, admin().to_string(), DORA_DEMON.to_string())
        .unwrap();

    let resp = contract
        .create_round(
            &mut app,
            creator(),
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
    assert_eq!(creator(), amaci_admin);

    let amaci_operator = maci_contract.amaci_query_operator(&app).unwrap();
    println!("{:?}", amaci_operator);
    assert_eq!(operator(), amaci_operator);

    let amaci_round_info = maci_contract.amaci_query_round_info(&app).unwrap();
    println!("{:?}", amaci_round_info);

    // Record creator balance after creating round
    let creator_balance_after = contract
        .balance_of(&app, creator().to_string(), DORA_DEMON.to_string())
        .unwrap();

    // Record admin balance after creating round
    let admin_balance_after = contract
        .balance_of(&app, admin().to_string(), DORA_DEMON.to_string())
        .unwrap();

    // Verify that creator balance decreased by small_base_payamount
    assert_eq!(
        creator_balance_before.amount - Uint128::from(small_base_payamount),
        creator_balance_after.amount
    );

    // Verify that admin balance did not change
    assert_eq!(admin_balance_before.amount, admin_balance_after.amount);

    let amaci_round_balance = contract
        .balance_of(
            &app,
            amaci_contract_addr.addr.to_string(),
            DORA_DEMON.to_string(),
        )
        .unwrap();
    let circuit_charge_config = contract.get_circuit_charge_config(&app).unwrap();
    let total_fee = Uint128::from(small_base_payamount);
    let operator_fee = Uint128::from(20000000000000000000u128);

    // Verify that contract balance is correct
    assert_eq!(Uint128::from(operator_fee), amaci_round_balance.amount); // Creating contract will transfer operator fee to the contract
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

    let creator_coin_amount = 200000000000000000000u128; // 200 DORA (covers 120 DORA total fee)
    let _operator_coin_amount = 1000000000000000000000u128; // 1000 DORA

    let mut app = cw_amaci::multitest::create_app();
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &creator(), coins(creator_coin_amount, DORA_DEMON))
            .unwrap();
    });

    let register_code_id = AmaciRegistryCodeId::store_code(&mut app);
    let amaci_code_id = MaciCodeId::store_default_code(&mut app);

    let label = "Dora AMaci Registry";
    let contract = register_code_id
        .instantiate(&mut app, creator(), amaci_code_id.id(), label)
        .unwrap();

    _ = contract.set_validators(&mut app, admin());

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

    let small_base_payamount = 120000000000000000000u128; // total small fee: 120 DORA (20 + 100)

    // Record balance before creating round
    let creator_balance_before = contract
        .balance_of(&app, creator().to_string(), DORA_DEMON.to_string())
        .unwrap();

    let admin_balance_before = contract
        .balance_of(&app, admin().to_string(), DORA_DEMON.to_string())
        .unwrap();

    let resp = contract
        .create_round_with_whitelist(
            &mut app,
            creator(),
            operator(),
            Uint256::from_u128(1u128),
            Uint256::from_u128(0u128),
            &coins(small_base_payamount, DORA_DEMON),
        )
        .unwrap();

    // Record balance after creating round
    let creator_balance_after = contract
        .balance_of(&app, creator().to_string(), DORA_DEMON.to_string())
        .unwrap();

    // Verify that creator balance decreased by small_base_payamount
    assert_eq!(
        creator_balance_before.amount - Uint128::from(small_base_payamount),
        creator_balance_after.amount
    );

    let amaci_contract_addr: InstantiationData = from_json(&resp.data.unwrap()).unwrap();
    println!("{:?}", amaci_contract_addr);
    let maci_contract = MaciContract::new(amaci_contract_addr.addr.clone());
    let amaci_admin = maci_contract.amaci_query_admin(&app).unwrap();
    println!("{:?}", amaci_admin);
    assert_eq!(creator(), amaci_admin);

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
    // let admin_fee = circuit_charge_config.fee_rate * total_fee;
    // Only 20 DORA goes to AMACI contract (instantiate fee)
    let operator_fee = Uint128::from(20000000000000000000u128);

    assert_eq!(Uint128::from(operator_fee), amaci_round_balance.amount);

    let num_sign_up = maci_contract.amaci_num_sign_up(&app).unwrap();
    assert_eq!(num_sign_up, Uint256::from_u128(0u128));

    let vote_option_map = maci_contract.amaci_vote_option_map(&app).unwrap();
    let max_vote_options = maci_contract.amaci_max_vote_options(&app).unwrap();
    assert_eq!(vote_option_map, vec!["", "", "", "", ""]);
    assert_eq!(max_vote_options, Uint256::from_u128(5u128));
    _ = maci_contract.amaci_set_vote_option_map(&mut app, creator());
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
    );

    _ = maci_contract.amaci_set_vote_option_map(&mut app, creator());

    app.update_block(next_block); // Start Voting
    let set_whitelist_only_in_pending = maci_contract
        .amaci_set_whitelist(&mut app, creator())
        .unwrap_err();
    assert_eq!(
        AmaciContractError::PeriodError {},
        set_whitelist_only_in_pending.downcast().unwrap()
    );
    let set_vote_option_map_error = maci_contract
        .amaci_set_vote_option_map(&mut app, creator())
        .unwrap_err();
    assert_eq!(
        AmaciContractError::PeriodError {},
        set_vote_option_map_error.downcast().unwrap()
    );

    let error_start_process_in_voting = maci_contract
        .amaci_start_process(&mut app, creator())
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
                        creator(),
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
                    a: data.proof.pi_a.to_string(),
                    b: data.proof.pi_b.to_string(),
                    c: data.proof.pi_c.to_string(),
                };

                println!("add_new_key proof {:?}", proof);
                _ = maci_contract
                    .amaci_add_key(&mut app, creator(), new_key_pub, nullifier, d, proof)
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

                _ = maci_contract.amaci_start_process(&mut app, creator());
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
                    .amaci_stop_processing(&mut app, creator())
                    .unwrap_err();
                assert_eq!(
                    AmaciContractError::MsgLeftProcess {},
                    error_stop_processing_with_not_finish_process
                        .downcast()
                        .unwrap()
                );

                let new_state_commitment = uint256_from_decimal_string(&data.new_state_commitment);
                let proof = Groth16ProofType {
                    a: data.proof.pi_a.to_string(),
                    b: data.proof.pi_b.to_string(),
                    c: data.proof.pi_c.to_string(),
                };
                println!("process_message proof {:?}", proof);
                println!(
                    "process_message new state commitment {:?}",
                    new_state_commitment
                );
                println!("------ processMessage ------");
                _ = maci_contract
                    .amaci_process_message(&mut app, creator(), new_state_commitment, proof)
                    .unwrap();
            }
            "processTally" => {
                let data: ProcessTallyData = deserialize_data(&entry.data);

                _ = maci_contract.amaci_stop_processing(&mut app, creator());
                println!(
                    "after stop process: {:?}",
                    maci_contract.amaci_get_period(&app).unwrap()
                );

                let error_start_process_in_talling = maci_contract
                    .amaci_start_process(&mut app, creator())
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
                    a: data.proof.pi_a.to_string(),
                    b: data.proof.pi_b.to_string(),
                    c: data.proof.pi_c.to_string(),
                };

                _ = maci_contract
                    .amaci_process_tally(&mut app, creator(), new_tally_commitment, tally_proof)
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

                let withdraw_error = maci_contract.amaci_claim(&mut app, creator()).unwrap_err();
                assert!(matches!(
                    withdraw_error.downcast::<AmaciContractError>().unwrap(),
                    AmaciContractError::UnauthorizedRegisty { .. }
                ));

                app.update_block(next_block_3_hours);
                _ = maci_contract.amaci_stop_tallying(&mut app, creator(), results, salt);

                let all_result = maci_contract.amaci_get_all_result(&app);
                println!("all_result: {:?}", all_result);
                let error_start_process = maci_contract
                    .amaci_start_process(&mut app, creator())
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

    let creator_balance_before_claim = contract
        .balance_of(&app, creator().to_string(), DORA_DEMON.to_string())
        .unwrap();
    println!(
        "creator_balance_before_claim: {:?}",
        creator_balance_before_claim
    );

    let operator_balance_before_claim = contract
        .balance_of(&app, operator().to_string(), DORA_DEMON.to_string())
        .unwrap();
    println!(
        "operator_balance_before_claim: {:?}",
        operator_balance_before_claim
    );

    // Trigger claim via registry contract (authorized caller)
    let _ = app
        .execute_contract(
            creator(),
            contract.addr(),
            &crate::msg::ExecuteMsg::Claim {
                round_addr: maci_contract.addr(),
            },
            &[],
        )
        .unwrap();
    let creator_balance = contract
        .balance_of(&app, creator().to_string(), DORA_DEMON.to_string())
        .unwrap();
    println!("creator_balance: {:?}", creator_balance);

    let operator_balance = contract
        .balance_of(&app, operator().to_string(), DORA_DEMON.to_string())
        .unwrap();
    println!("operator_balance: {:?}", operator_balance);

    let admin_balance = contract
        .balance_of(&app, admin().to_string(), DORA_DEMON.to_string())
        .unwrap();
    println!("admin_balance: {:?}", admin_balance);

    let round_balance_after_claim = contract
        .balance_of(
            &app,
            maci_contract.addr().to_string(),
            DORA_DEMON.to_string(),
        )
        .unwrap();
    println!("round_balance_after_claim: {:?}", round_balance_after_claim);

    let total_amount = Uint128::from(round_balance_before_claim.amount);
    let admin_fee = circuit_charge_config.fee_rate * total_amount;
    println!("admin_fee: {:?}", admin_fee);
    let claim_amount = total_amount - admin_fee;
    println!("claim_amount: {:?}", claim_amount);
    let operator_reward = claim_amount.multiply_ratio(100u128 - (50u128 + 5u128 * 2), 100u128);
    let penalty_amount = claim_amount - operator_reward;
    println!("operator_reward: {:?}", operator_reward);
    println!("penalty_amount: {:?}", penalty_amount);

    assert_eq!(
        operator_balance.amount,
        operator_reward + operator_balance_before_claim.amount // Uint128::from(0u128) // after 4 days, operator reward is 0, all funds are returned to creator
    );

    assert_eq!(admin_balance.amount, admin_fee);

    assert_eq!(
        creator_balance.amount,
        penalty_amount + creator_balance_before_claim.amount
    );

    assert_eq!(
        round_balance_after_claim.amount,
        round_balance_before_claim.amount - claim_amount - admin_fee
    );

    assert_eq!(round_balance_after_claim.amount, Uint128::from(0u128));

    let claim_error = maci_contract.amaci_claim(&mut app, creator()).unwrap_err();
    assert!(matches!(
        claim_error.downcast::<AmaciContractError>().unwrap(),
        AmaciContractError::UnauthorizedRegisty { .. }
    ));

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

    let creator_coin_amount = 200000000000000000000u128; // 200 DORA (covers 120 DORA total fee)
    let _operator_coin_amount = 1000000000000000000000u128; // 1000 DORA

    let mut app = cw_amaci::multitest::create_app();
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &creator(), coins(creator_coin_amount, DORA_DEMON))
            .unwrap();
    });

    let register_code_id = AmaciRegistryCodeId::store_code(&mut app);
    let amaci_code_id = MaciCodeId::store_default_code(&mut app);

    let label = "Dora AMaci Registry";
    let contract = register_code_id
        .instantiate(&mut app, creator(), amaci_code_id.id(), label)
        .unwrap();

    _ = contract.set_validators(&mut app, admin());

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

    let small_base_payamount = 120000000000000000000u128; // total small fee: 120 DORA (20 + 100)

    // Record balance before creating the round
    let creator_balance_before = contract
        .balance_of(&app, creator().to_string(), DORA_DEMON.to_string())
        .unwrap();

    let resp = contract
        .create_round_with_whitelist(
            &mut app,
            creator(),
            operator(),
            Uint256::from_u128(1u128),
            Uint256::from_u128(0u128),
            &coins(small_base_payamount, DORA_DEMON),
        )
        .unwrap();

    // Record balance after creating the round
    let creator_balance_after = contract
        .balance_of(&app, creator().to_string(), DORA_DEMON.to_string())
        .unwrap();

    // Verify that the creator's balance decreased by small_base_payamount
    assert_eq!(
        creator_balance_before.amount - Uint128::from(small_base_payamount),
        creator_balance_after.amount
    );

    let amaci_contract_addr: InstantiationData = from_json(&resp.data.unwrap()).unwrap();
    println!("{:?}", amaci_contract_addr);
    let maci_contract = MaciContract::new(amaci_contract_addr.addr.clone());
    let amaci_admin = maci_contract.amaci_query_admin(&app).unwrap();
    println!("{:?}", amaci_admin);
    assert_eq!(creator(), amaci_admin);

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
    // Only 20 DORA goes to AMACI contract (instantiate fee)
    let operator_fee = Uint128::from(20000000000000000000u128);

    assert_eq!(Uint128::from(operator_fee), amaci_round_balance.amount);

    let num_sign_up = maci_contract.amaci_num_sign_up(&app).unwrap();
    assert_eq!(num_sign_up, Uint256::from_u128(0u128));

    let vote_option_map = maci_contract.amaci_vote_option_map(&app).unwrap();
    let max_vote_options = maci_contract.amaci_max_vote_options(&app).unwrap();
    assert_eq!(vote_option_map, vec!["", "", "", "", ""]);
    assert_eq!(max_vote_options, Uint256::from_u128(5u128));
    _ = maci_contract.amaci_set_vote_option_map(&mut app, creator());
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
    );

    _ = maci_contract.amaci_set_vote_option_map(&mut app, creator());

    app.update_block(next_block); // Start Voting
    let set_whitelist_only_in_pending = maci_contract
        .amaci_set_whitelist(&mut app, creator())
        .unwrap_err();
    assert_eq!(
        AmaciContractError::PeriodError {},
        set_whitelist_only_in_pending.downcast().unwrap()
    );
    let set_vote_option_map_error = maci_contract
        .amaci_set_vote_option_map(&mut app, creator())
        .unwrap_err();
    assert_eq!(
        AmaciContractError::PeriodError {},
        set_vote_option_map_error.downcast().unwrap()
    );

    let error_start_process_in_voting = maci_contract
        .amaci_start_process(&mut app, creator())
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
                        creator(),
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
                    a: data.proof.pi_a.to_string(),
                    b: data.proof.pi_b.to_string(),
                    c: data.proof.pi_c.to_string(),
                };

                println!("add_new_key proof {:?}", proof);
                _ = maci_contract
                    .amaci_add_key(&mut app, creator(), new_key_pub, nullifier, d, proof)
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

                _ = maci_contract.amaci_start_process(&mut app, creator());
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
                    .amaci_stop_processing(&mut app, creator())
                    .unwrap_err();
                assert_eq!(
                    AmaciContractError::MsgLeftProcess {},
                    error_stop_processing_with_not_finish_process
                        .downcast()
                        .unwrap()
                );

                let new_state_commitment = uint256_from_decimal_string(&data.new_state_commitment);
                let proof = Groth16ProofType {
                    a: data.proof.pi_a.to_string(),
                    b: data.proof.pi_b.to_string(),
                    c: data.proof.pi_c.to_string(),
                };
                println!("process_message proof {:?}", proof);
                println!(
                    "process_message new state commitment {:?}",
                    new_state_commitment
                );
                println!("------ processMessage ------");
                _ = maci_contract
                    .amaci_process_message(&mut app, creator(), new_state_commitment, proof)
                    .unwrap();
            }
            "processTally" => {
                let data: ProcessTallyData = deserialize_data(&entry.data);

                _ = maci_contract.amaci_stop_processing(&mut app, creator());
                println!(
                    "after stop process: {:?}",
                    maci_contract.amaci_get_period(&app).unwrap()
                );

                let error_start_process_in_talling = maci_contract
                    .amaci_start_process(&mut app, creator())
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
                    a: data.proof.pi_a.to_string(),
                    b: data.proof.pi_b.to_string(),
                    c: data.proof.pi_c.to_string(),
                };

                _ = maci_contract
                    .amaci_process_tally(&mut app, creator(), new_tally_commitment, tally_proof)
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

                let withdraw_error = maci_contract.amaci_claim(&mut app, creator()).unwrap_err();
                assert!(matches!(
                    withdraw_error.downcast::<AmaciContractError>().unwrap(),
                    AmaciContractError::UnauthorizedRegisty { .. }
                ));

                app.update_block(next_block_3_hours);
                _ = maci_contract.amaci_stop_tallying(&mut app, creator(), results, salt);

                let all_result = maci_contract.amaci_get_all_result(&app);
                println!("all_result: {:?}", all_result);
                let error_start_process = maci_contract
                    .amaci_start_process(&mut app, creator())
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

    let creator_balance_before_claim = contract
        .balance_of(&app, creator().to_string(), DORA_DEMON.to_string())
        .unwrap();
    println!(
        "creator_balance_before_claim: {:?}",
        creator_balance_before_claim
    );

    let operator_balance_before_claim = contract
        .balance_of(&app, operator().to_string(), DORA_DEMON.to_string())
        .unwrap();
    println!(
        "operator_balance_before_claim: {:?}",
        operator_balance_before_claim
    );

    app.update_block(next_block_4_days); // after 4 days, operator reward is 0, all funds are returned to creator
    let _ = app
        .execute_contract(
            creator(),
            contract.addr(),
            &crate::msg::ExecuteMsg::Claim {
                round_addr: maci_contract.addr(),
            },
            &[],
        )
        .unwrap();
    let creator_balance = contract
        .balance_of(&app, creator().to_string(), DORA_DEMON.to_string())
        .unwrap();
    println!("creator_balance: {:?}", creator_balance);

    let operator_balance = contract
        .balance_of(&app, operator().to_string(), DORA_DEMON.to_string())
        .unwrap();
    println!("operator_balance: {:?}", operator_balance);

    let admin_balance = contract
        .balance_of(&app, admin().to_string(), DORA_DEMON.to_string())
        .unwrap();
    println!("admin_balance: {:?}", admin_balance);

    let round_balance_after_claim = contract
        .balance_of(
            &app,
            maci_contract.addr().to_string(),
            DORA_DEMON.to_string(),
        )
        .unwrap();
    println!("round_balance_after_claim: {:?}", round_balance_after_claim);

    let total_amount = Uint128::from(round_balance_before_claim.amount);
    // let admin_fee = circuit_charge_config.fee_rate * total_amount;
    let admin_fee = Uint128::from(0u128);
    let claim_amount = total_amount;
    println!("claim_amount: {:?}", claim_amount);
    let operator_reward = claim_amount.multiply_ratio(100u128 - (50u128 + 5u128 * 1), 100u128);
    let operator_reward = Uint128::from(0u128); // after 4 days, operator reward is 0, all funds are returned to creator
    let penalty_amount = claim_amount - operator_reward;
    println!("operator_reward: {:?}", operator_reward);
    println!("penalty_amount: {:?}", penalty_amount);

    assert_eq!(
        operator_balance.amount,
        Uint128::from(0u128) // after 4 days, operator reward is 0, all funds are returned to creator
    );

    assert_eq!(admin_balance.amount, admin_fee);

    assert_eq!(
        creator_balance.amount,
        penalty_amount + creator_balance_before_claim.amount
    );

    assert_eq!(
        round_balance_after_claim.amount,
        round_balance_before_claim.amount - claim_amount - admin_fee
    );

    assert_eq!(round_balance_after_claim.amount, Uint128::from(0u128));

    let claim_error = maci_contract.amaci_claim(&mut app, creator()).unwrap_err();
    assert!(matches!(
        claim_error.downcast::<AmaciContractError>().unwrap(),
        AmaciContractError::UnauthorizedRegisty { .. }
    ));

    let tally_delay = maci_contract.amaci_query_tally_delay(&app).unwrap();
    println!("tally_delay: {:?}", tally_delay);
}

// ========== Moved from claim_create_tests.rs ==========

#[test]
fn create_round_total_fee_and_transfers() {
    let mut app = create_app_dora();
    // fund creator
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &creator(), coins(200000000000000000000u128, DORA_DEMON))
            .unwrap();
    });

    // Store AMACI code directly into this app
    let amaci_wrapper = cw_multi_test::ContractWrapper::new(
        cw_amaci::contract::execute,
        cw_amaci::contract::instantiate,
        cw_amaci::contract::query,
    );
    let amaci_code_id = app.store_code(Box::new(amaci_wrapper));

    let registry = instantiate_registry_dora(&mut app, amaci_code_id);
    setup_operator_dora(&mut app, &registry);

    let start = Timestamp::from_nanos(1_571_797_424_879_000_000);
    let end = start.plus_minutes(21);
    let whitelist = Some(cw_amaci::msg::WhitelistBase {
        users: vec![
            cw_amaci::msg::WhitelistBaseConfig { addr: user1() },
            cw_amaci::msg::WhitelistBaseConfig { addr: user2() },
        ],
    });
    let msg = crate::msg::ExecuteMsg::CreateRound {
        operator: operator(),
        max_voter: Uint256::from_u128(5),
        max_option: Uint256::from_u128(5),
        voice_credit_amount: Uint256::from_u128(30),
        round_info: cw_amaci::state::RoundInfo { title: "HackWasm Berlin".into(), description: "Hack In Brelin".into(), link: "https://baidu.com".into() },
        voting_time: cw_amaci::state::VotingTime { start_time: start, end_time: end },
        whitelist,
        pre_deactivate_root: Uint256::from_u128(0),
        circuit_type: Uint256::from_u128(1),
        certification_system: Uint256::from_u128(0),
    };

    // total_fee = 120 DORA for small case
    let resp = app
        .execute_contract(
            creator(),
            registry.clone(),
            &msg,
            &coins(120000000000000000000u128, DORA_DEMON),
        )
        .unwrap();

    let data: super::InstantiationData = cosmwasm_std::from_json(resp.data.clone().unwrap()).unwrap();
    let amaci_addr = data.addr;

    // Check balances
    let amaci_balance = app
        .wrap()
        .query_balance(amaci_addr.to_string(), DORA_DEMON.to_string())
        .unwrap();
    assert_eq!(amaci_balance.amount, Uint128::from(20000000000000000000u128));

    let sponsor_addr = find_attr(&resp, "sponsor_address").unwrap();
    let sponsor_balance = app
        .wrap()
        .query_balance(sponsor_addr.clone(), DORA_DEMON.to_string())
        .unwrap();
    assert_eq!(sponsor_balance.amount, Uint128::from(100000000000000000000u128));

    // Advance time to timeout and claim
    let four_days_ns = 4u64 * 24 * 60 * 60 * 1_000_000_000u64;
    let claim_time = end.nanos() + four_days_ns + 1;
    app.update_block(|b| b.time = Timestamp::from_nanos(claim_time));

    let creator_before = app
        .wrap()
        .query_balance(creator().to_string(), DORA_DEMON.to_string())
        .unwrap();

    let _claim_resp = app
        .execute_contract(
            creator(),
            registry.clone(),
            &crate::msg::ExecuteMsg::Claim { round_addr: amaci_addr.clone() },
            &[],
        )
        .unwrap();

    // SponsorStargate should have transferred sponsor funds back to creator
    let sponsor_after = app
        .wrap()
        .query_balance(sponsor_addr, DORA_DEMON.to_string())
        .unwrap();
    assert_eq!(sponsor_after.amount, Uint128::zero());

    // Creator should receive sponsor 100 + amaci 20 = 120 DORA
    let creator_after = app
        .wrap()
        .query_balance(creator().to_string(), DORA_DEMON.to_string())
        .unwrap();
    assert_eq!(
        creator_after.amount,
        creator_before.amount + Uint128::from(120000000000000000000u128)
    );
}

#[test]
fn create_round_medium_total_fee_and_transfers() {
    let mut app = create_app_dora();
    // fund creator
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &creator(), coins(2000000000000000000000u128, DORA_DEMON))
            .unwrap();
    });

    // Store AMACI code directly into this app
    let amaci_wrapper = cw_multi_test::ContractWrapper::new(
        cw_amaci::contract::execute,
        cw_amaci::contract::instantiate,
        cw_amaci::contract::query,
    );
    let amaci_code_id = app.store_code(Box::new(amaci_wrapper));

    let registry = instantiate_registry_dora(&mut app, amaci_code_id);
    setup_operator_dora(&mut app, &registry);

    let start = Timestamp::from_nanos(1_571_797_424_879_000_000);
    let end = start.plus_minutes(21);
    let whitelist = Some(cw_amaci::msg::WhitelistBase {
        users: vec![
            cw_amaci::msg::WhitelistBaseConfig { addr: user1() },
            cw_amaci::msg::WhitelistBaseConfig { addr: user2() },
        ],
    });
    let msg = crate::msg::ExecuteMsg::CreateRound {
        operator: operator(),
        max_voter: Uint256::from_u128(625),
        max_option: Uint256::from_u128(25),
        voice_credit_amount: Uint256::from_u128(30),
        round_info: cw_amaci::state::RoundInfo {
            title: "HackWasm Berlin".into(),
            description: "Hack In Brelin".into(),
            link: "https://baidu.com".into(),
        },
        voting_time: cw_amaci::state::VotingTime {
            start_time: start,
            end_time: end,
        },
        whitelist,
        pre_deactivate_root: Uint256::from_u128(0),
        circuit_type: Uint256::from_u128(1),
        certification_system: Uint256::from_u128(0),
    };

    // total_fee = 950 DORA for medium case (750 + 200)
    let resp = app
        .execute_contract(
            creator(),
            registry.clone(),
            &msg,
            &coins(950000000000000000000u128, DORA_DEMON),
        )
        .unwrap();

    let data: super::InstantiationData = cosmwasm_std::from_json(resp.data.clone().unwrap()).unwrap();
    let amaci_addr = data.addr;

    // AMACI should have 750 DORA
    let amaci_balance = app
        .wrap()
        .query_balance(amaci_addr.to_string(), DORA_DEMON.to_string())
        .unwrap();
    assert_eq!(
        amaci_balance.amount,
        Uint128::from(750000000000000000000u128)
    );

    // Sponsor address should have received 200 DORA
    let sponsor_addr = find_attr(&resp, "sponsor_address").unwrap();
    let sponsor_balance = app
        .wrap()
        .query_balance(sponsor_addr.clone(), DORA_DEMON.to_string())
        .unwrap();
    assert_eq!(
        sponsor_balance.amount,
        Uint128::from(200000000000000000000u128)
    );

    // Advance to timeout and claim
    let four_days_ns = 4u64 * 24 * 60 * 60 * 1_000_000_000u64;
    let claim_time = end.nanos() + four_days_ns + 1;
    app.update_block(|b| b.time = Timestamp::from_nanos(claim_time));

    let creator_before = app
        .wrap()
        .query_balance(creator().to_string(), DORA_DEMON.to_string())
        .unwrap();

    let _ = app
        .execute_contract(
            creator(),
            registry.clone(),
            &crate::msg::ExecuteMsg::Claim {
                round_addr: amaci_addr.clone(),
            },
            &[],
        )
        .unwrap();

    // Sponsor funds moved to creator
    let sponsor_after = app
        .wrap()
        .query_balance(sponsor_addr, DORA_DEMON.to_string())
        .unwrap();
    assert_eq!(sponsor_after.amount, Uint128::zero());

    // Creator received 750 + 200 = 950 DORA
    let creator_after = app
        .wrap()
        .query_balance(creator().to_string(), DORA_DEMON.to_string())
        .unwrap();
    assert_eq!(
        creator_after.amount,
        creator_before.amount + Uint128::from(950000000000000000000u128)
    );
}

#[test]
fn create_round_medium_insufficient_fee_error() {
    let mut app = create_app_dora();
    // fund creator
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &creator(), coins(2000000000000000000000u128, DORA_DEMON))
            .unwrap();
    });

    // Store AMACI code directly into this app
    let amaci_wrapper = cw_multi_test::ContractWrapper::new(
        cw_amaci::contract::execute,
        cw_amaci::contract::instantiate,
        cw_amaci::contract::query,
    );
    let amaci_code_id = app.store_code(Box::new(amaci_wrapper));

    let registry = instantiate_registry_dora(&mut app, amaci_code_id);
    setup_operator_dora(&mut app, &registry);

    let start = Timestamp::from_nanos(1_571_797_424_879_000_000);
    let end = start.plus_minutes(21);
    let whitelist = Some(cw_amaci::msg::WhitelistBase {
        users: vec![
            cw_amaci::msg::WhitelistBaseConfig { addr: user1() },
            cw_amaci::msg::WhitelistBaseConfig { addr: user2() },
        ],
    });
    let msg = crate::msg::ExecuteMsg::CreateRound {
        operator: operator(),
        max_voter: Uint256::from_u128(625),
        max_option: Uint256::from_u128(25),
        voice_credit_amount: Uint256::from_u128(30),
        round_info: cw_amaci::state::RoundInfo {
            title: "HackWasm Berlin".into(),
            description: "Hack In Brelin".into(),
            link: "https://baidu.com".into(),
        },
        voting_time: cw_amaci::state::VotingTime {
            start_time: start,
            end_time: end,
        },
        whitelist,
        pre_deactivate_root: Uint256::from_u128(0),
        circuit_type: Uint256::from_u128(1),
        certification_system: Uint256::from_u128(0),
    };

    // Medium total fee required is 950 DORA (750 + 200); provide less to trigger error
    let provided = 900000000000000000000u128; // 900 DORA
    let result = app.execute_contract(
        creator(),
        registry.clone(),
        &msg,
        &coins(provided, DORA_DEMON),
    );

    assert!(result.is_err(), "Should fail due to insufficient fee");
    let err = result.unwrap_err().downcast::<RegistryContractError>().unwrap();
    assert_eq!(
        err,
        RegistryContractError::InsufficientFee {
            required: Uint128::from(950000000000000000000u128),
            provided: Uint128::from(provided),
        }
    );
}

// ========== Additional coverage for CreateRound and Claim ==========

#[test]
fn create_round_without_pubkey_should_fail() {
    let admin_coin_amount = 1000000000000000000000u128; // 1000 DORA
    let creator_coin_amount = 1000000000000000000000u128; // 1000 DORA

    let mut app = cw_amaci::multitest::create_app();
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &admin(), coins(admin_coin_amount, DORA_DEMON))
            .unwrap();
        router
            .bank
            .init_balance(storage, &creator(), coins(creator_coin_amount, DORA_DEMON))
            .unwrap();
    });

    let register_code_id = super::AmaciRegistryCodeId::store_code(&mut app);
    let amaci_code_id = cw_amaci::multitest::MaciCodeId::store_default_code(&mut app);
    let contract = register_code_id
        .instantiate(&mut app, creator(), amaci_code_id.id(), "Dora AMaci Registry")
        .unwrap();

    let _ = contract.set_validators(&mut app, admin());
    let _ = contract.set_maci_operator(&mut app, user1(), operator());
    // Intentionally NOT setting operator pubkey

    // Attempt small round with full total fee 120 DORA
    let res = contract
        .create_round(
            &mut app,
            creator(),
            operator(),
            Uint256::from_u128(1u128),
            Uint256::from_u128(0u128),
            &coins(120000000000000000000u128, DORA_DEMON),
        )
        .unwrap_err();

    assert_eq!(RegistryContractError::NotSetOperatorPubkey, res.downcast().unwrap());
}

#[test]
fn create_round_large_size_should_fail_no_matched_size_circuit() {
    let admin_coin_amount = 1000000000000000000000u128; // 1000 DORA
    let creator_coin_amount = 1000000000000000000000u128; // 1000 DORA

    let mut app = cw_amaci::multitest::create_app();
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &admin(), coins(admin_coin_amount, DORA_DEMON))
            .unwrap();
        router
            .bank
            .init_balance(storage, &creator(), coins(creator_coin_amount, DORA_DEMON))
            .unwrap();
    });
    let register_code_id = super::AmaciRegistryCodeId::store_code(&mut app);
    let amaci_code_id = cw_amaci::multitest::MaciCodeId::store_default_code(&mut app);
    let contract = register_code_id
        .instantiate(&mut app, creator(), amaci_code_id.id(), "Dora AMaci Registry")
        .unwrap();

    let _ = contract.set_validators(&mut app, admin());
    let _ = contract.set_maci_operator(&mut app, user1(), operator());
    let _ = contract.set_maci_operator_pubkey(&mut app, operator(), operator_pubkey1());

    // Directly execute with oversized max_voter to hit NoMatchedSizeCircuit
    let round_info = cw_amaci::state::RoundInfo {
        title: String::from("HackWasm Berlin"),
        description: String::from("Hack In Brelin"),
        link: String::from("https://baidu.com"),
    };
    let start_time = Timestamp::from_nanos(1571797424879000000);
    let end_time = start_time.plus_minutes(21);
    let msg = crate::msg::ExecuteMsg::CreateRound {
        operator: operator(),
        round_info,
        max_voter: Uint256::from_u128(626u128), // exceed medium bound
        max_option: Uint256::from_u128(5u128),
        voice_credit_amount: Uint256::from_u128(30u128),
        voting_time: cw_amaci::state::VotingTime { start_time, end_time },
        whitelist: None,
        pre_deactivate_root: Uint256::from_u128(0u128),
        circuit_type: Uint256::from_u128(1u128),
        certification_system: Uint256::from_u128(0u128),
    };

    let err = app
        .execute_contract(creator(), contract.addr(), &msg, &coins(120000000000000000000u128, DORA_DEMON))
        .unwrap_err();
    assert_eq!(RegistryContractError::NoMatchedSizeCircuit, err.downcast().unwrap());
}

#[test]
fn create_round_small_insufficient_fee_should_fail() {
    let admin_coin_amount = 1000000000000000000000u128; // 1000 DORA
    let creator_coin_amount = 1000000000000000000000u128; // 1000 DORA

    let mut app = cw_amaci::multitest::create_app();
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &admin(), coins(admin_coin_amount, DORA_DEMON))
            .unwrap();
        router
            .bank
            .init_balance(storage, &creator(), coins(creator_coin_amount, DORA_DEMON))
            .unwrap();
    });
    let register_code_id = super::AmaciRegistryCodeId::store_code(&mut app);
    let amaci_code_id = cw_amaci::multitest::MaciCodeId::store_default_code(&mut app);
    let contract = register_code_id
        .instantiate(&mut app, creator(), amaci_code_id.id(), "Dora AMaci Registry")
        .unwrap();

    let _ = contract.set_validators(&mut app, admin());
    let _ = contract.set_maci_operator(&mut app, user1(), operator());
    let _ = contract.set_maci_operator_pubkey(&mut app, operator(), operator_pubkey1());

    // Use helper to create small round but provide only 100 DORA (< 120)
    let err = contract
        .create_round(
            &mut app,
            creator(),
            operator(),
            Uint256::from_u128(1u128),
            Uint256::from_u128(0u128),
            &coins(100000000000000000000u128, DORA_DEMON),
        )
        .unwrap_err();

    assert_eq!(
        RegistryContractError::InsufficientFee {
            required: Uint128::from(120000000000000000000u128),
            provided: Uint128::from(100000000000000000000u128),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn create_round_invalid_operator_address_should_fail() {
    let admin_coin_amount = 1000000000000000000000u128; // 1000 DORA
    let creator_coin_amount = 1000000000000000000000u128; // 1000 DORA

    let mut app = cw_amaci::multitest::create_app();
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &admin(), coins(admin_coin_amount, DORA_DEMON))
            .unwrap();
        router
            .bank
            .init_balance(storage, &creator(), coins(creator_coin_amount, DORA_DEMON))
            .unwrap();
    });
    let register_code_id = super::AmaciRegistryCodeId::store_code(&mut app);
    let amaci_code_id = cw_amaci::multitest::MaciCodeId::store_default_code(&mut app);
    let contract = register_code_id
        .instantiate(&mut app, creator(), amaci_code_id.id(), "Dora AMaci Registry")
        .unwrap();

    let _ = contract.set_validators(&mut app, admin());
    // Do not set operator pubkey to ensure we hit address validation first

    let round_info = cw_amaci::state::RoundInfo {
        title: String::from("HackWasm Berlin"),
        description: String::from("Hack In Brelin"),
        link: String::from("https://baidu.com"),
    };
    let start_time = Timestamp::from_nanos(1571797424879000000);
    let end_time = start_time.plus_minutes(21);
    let invalid_operator = Addr::unchecked("badaddr");
    let msg = crate::msg::ExecuteMsg::CreateRound {
        operator: invalid_operator,
        round_info,
        max_voter: Uint256::from_u128(3u128),
        max_option: Uint256::from_u128(5u128),
        voice_credit_amount: Uint256::from_u128(30u128),
        voting_time: cw_amaci::state::VotingTime { start_time, end_time },
        whitelist: None,
        pre_deactivate_root: Uint256::from_u128(0u128),
        circuit_type: Uint256::from_u128(1u128),
        certification_system: Uint256::from_u128(0u128),
    };

    let err = app
        .execute_contract(creator(), contract.addr(), &msg, &coins(120000000000000000000u128, DORA_DEMON))
        .unwrap_err();
    assert_eq!(
        RegistryContractError::InvalidAddress { address: "badaddr".into() },
        err.downcast().unwrap()
    );
}

#[test]
fn claim_twice_via_registry_should_fail() {
    // Reuse medium case to ensure non-zero AMACI balance and sponsor funds
    let mut app = create_app_dora();
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &creator(), coins(2000000000000000000000u128, DORA_DEMON))
            .unwrap();
    });
    let amaci_wrapper = cw_multi_test::ContractWrapper::new(
        cw_amaci::contract::execute,
        cw_amaci::contract::instantiate,
        cw_amaci::contract::query,
    );
    let amaci_code_id = app.store_code(Box::new(amaci_wrapper));
    let registry = instantiate_registry_dora(&mut app, amaci_code_id);
    setup_operator_dora(&mut app, &registry);

    let start = Timestamp::from_nanos(1_571_797_424_879_000_000);
    let end = start.plus_minutes(21);
    let msg = crate::msg::ExecuteMsg::CreateRound {
        operator: operator(),
        max_voter: Uint256::from_u128(625),
        max_option: Uint256::from_u128(25),
        voice_credit_amount: Uint256::from_u128(30),
        round_info: cw_amaci::state::RoundInfo { title: "HackWasm Berlin".into(), description: "Hack In Brelin".into(), link: "https://baidu.com".into() },
        voting_time: cw_amaci::state::VotingTime { start_time: start, end_time: end },
        whitelist: Some(cw_amaci::msg::WhitelistBase { users: vec![
            cw_amaci::msg::WhitelistBaseConfig { addr: user1() },
            cw_amaci::msg::WhitelistBaseConfig { addr: user2() },
        ]}),
        pre_deactivate_root: Uint256::from_u128(0),
        circuit_type: Uint256::from_u128(1),
        certification_system: Uint256::from_u128(0),
    };
    let resp = app
        .execute_contract(
            creator(),
            registry.clone(),
            &msg,
            &coins(950000000000000000000u128, DORA_DEMON),
        )
        .unwrap();
    let data: super::InstantiationData = cosmwasm_std::from_json(resp.data.clone().unwrap()).unwrap();
    let amaci_addr = data.addr;

    // Fast-forward and claim once
    let four_days_ns = 4u64 * 24 * 60 * 60 * 1_000_000_000u64;
    let claim_time = end.nanos() + four_days_ns + 1;
    app.update_block(|b| b.time = Timestamp::from_nanos(claim_time));
    let _ = app
        .execute_contract(
            creator(),
            registry.clone(),
            &crate::msg::ExecuteMsg::Claim { round_addr: amaci_addr.clone() },
            &[],
        )
        .unwrap();

    // Second claim via registry should fail (AMACI balance drained)
    let err = app
        .execute_contract(
            creator(),
            registry.clone(),
            &crate::msg::ExecuteMsg::Claim { round_addr: amaci_addr },
            &[],
        )
        .unwrap_err();
    // Should error (AMACI has zero funds after first claim)
    assert!(err.to_string().contains("Error executing WasmMsg"));
}

#[test]
fn create_round_small_early_claim_via_registry_during_voting_should_fail() {
    let admin_coin_amount = 1000000000000000000000u128; // 1000 DORA
    let creator_coin_amount = 1000000000000000000000u128; // 1000 DORA

    let mut app = cw_amaci::multitest::create_app();
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &admin(), coins(admin_coin_amount, DORA_DEMON))
            .unwrap();
        router
            .bank
            .init_balance(storage, &creator(), coins(creator_coin_amount, DORA_DEMON))
            .unwrap();
    });

    let register_code_id = super::AmaciRegistryCodeId::store_code(&mut app);
    let amaci_code_id = cw_amaci::multitest::MaciCodeId::store_default_code(&mut app);
    let contract = register_code_id
        .instantiate(&mut app, creator(), amaci_code_id.id(), "Dora AMaci Registry")
        .unwrap();

    let _ = contract.set_validators(&mut app, admin());
    let _ = contract.set_maci_operator(&mut app, user1(), operator());
    let _ = contract.set_maci_operator_pubkey(&mut app, operator(), operator_pubkey1());

    // Create small round (total fee 120 DORA)
    let resp = contract
        .create_round_with_whitelist(
            &mut app,
            creator(),
            operator(),
            Uint256::from_u128(1u128),
            Uint256::from_u128(0u128),
            &coins(120000000000000000000u128, DORA_DEMON),
        )
        .unwrap();
    let data: super::InstantiationData = from_json(&resp.data.unwrap()).unwrap();
    let maci_contract = MaciContract::new(data.addr.clone());

    // Move time into voting window (start + 60s)
    let voting_time = maci_contract.amaci_get_voting_time(&app).unwrap();
    let t = voting_time.start_time.plus_seconds(60);
    app.update_block(|b| b.time = t);

    // Early claim via registry should fail with PeriodError bubbled up from AMACI
    let err = app
        .execute_contract(
            creator(),
            contract.addr(),
            &crate::msg::ExecuteMsg::Claim { round_addr: data.addr.clone() },
            &[],
        )
        .unwrap_err();
    // PeriodError propagated from AMACI claim via Wasm execute
    assert!(err.to_string().contains("Error executing WasmMsg"));
}

#[test]
fn create_round_small_early_claim_via_registry_during_tallying_should_fail() {
    let admin_coin_amount = 1000000000000000000000u128; // 1000 DORA
    let creator_coin_amount = 1000000000000000000000u128; // 1000 DORA

    let mut app = cw_amaci::multitest::create_app();
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &admin(), coins(admin_coin_amount, DORA_DEMON))
            .unwrap();
        router
            .bank
            .init_balance(storage, &creator(), coins(creator_coin_amount, DORA_DEMON))
            .unwrap();
    });

    let register_code_id = super::AmaciRegistryCodeId::store_code(&mut app);
    let amaci_code_id = cw_amaci::multitest::MaciCodeId::store_default_code(&mut app);
    let contract = register_code_id
        .instantiate(&mut app, creator(), amaci_code_id.id(), "Dora AMaci Registry")
        .unwrap();

    let _ = contract.set_validators(&mut app, admin());
    let _ = contract.set_maci_operator(&mut app, user1(), operator());
    let _ = contract.set_maci_operator_pubkey(&mut app, operator(), operator_pubkey1());

    // Create small round (total fee 120 DORA)
    let resp = contract
        .create_round_with_whitelist(
            &mut app,
            creator(),
            operator(),
            Uint256::from_u128(1u128),
            Uint256::from_u128(0u128),
            &coins(120000000000000000000u128, DORA_DEMON),
        )
        .unwrap();
    let data: super::InstantiationData = from_json(&resp.data.unwrap()).unwrap();
    let maci_contract = MaciContract::new(data.addr.clone());

    // Move time into tally window but before timeout (end + 60s)
    let voting_time = maci_contract.amaci_get_voting_time(&app).unwrap();
    let t = voting_time.end_time.plus_seconds(60);
    app.update_block(|b| b.time = t);

    // Early claim via registry should fail with PeriodError
    let err = app
        .execute_contract(
            creator(),
            contract.addr(),
            &crate::msg::ExecuteMsg::Claim { round_addr: data.addr.clone() },
            &[],
        )
        .unwrap_err();
    assert!(err.to_string().contains("Error executing WasmMsg"));
}
