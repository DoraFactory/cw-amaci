use cosmwasm_std::{coins, from_json};
use cw_multi_test::App;

// use crate::error::ContractError;
// use crate::msg::ClaimsResponse;
use crate::{
    multitest::{
        operator, operator2, operator3, owner, pubkey1, pubkey2, pubkey3, user1, user2, user3,
        user4, AmaciRegistryCodeId, InstantiationData, DORA_DEMON,
    },
    state::ValidatorSet,
    ContractError,
};
use cw_amaci::multitest::{MaciCodeId, MaciContract};
use cw_amaci::ContractError as AmaciContractError;
#[test]
fn instantiate_should_works() {
    let user1_coin_amount = 30u128;
    let user2_coin_amount = 20u128;
    let user3_coin_amount = 10u128;

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &user1(), coins(user1_coin_amount, DORA_DEMON))
            .unwrap();
        router
            .bank
            .init_balance(storage, &user2(), coins(user2_coin_amount, DORA_DEMON))
            .unwrap();

        router
            .bank
            .init_balance(storage, &user3(), coins(user3_coin_amount, DORA_DEMON))
            .unwrap();
        // router
        //     .bank
        //     .init_balance(storage, &(), coins(500, ARCH_DEMON))
        //     .unwrap();
    });

    let code_id = AmaciRegistryCodeId::store_code(&mut app);
    let label = "Dora AMaci Registry";
    let contract = code_id.instantiate(&mut app, owner(), 1u64, label).unwrap();

    let not_admin_or_operator_set_validators =
        contract.set_validators(&mut app, user1()).unwrap_err();
    assert_eq!(
        ContractError::Unauthorized {},
        not_admin_or_operator_set_validators.downcast().unwrap()
    );

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

    _ = contract.set_maci_operator_pubkey(&mut app, operator(), pubkey1());
    let user1_operator_pubkey = contract.get_operator_pubkey(&app, operator()).unwrap();
    assert_eq!(pubkey1(), user1_operator_pubkey);

    _ = contract.remove_validator(&mut app, owner(), user4());
    let validator_set_after_remove_user4 = contract.get_validators(&app).unwrap();
    assert_eq!(
        ValidatorSet {
            addresses: vec![user1(), user2()]
        },
        validator_set_after_remove_user4
    );

    let not_validator_set_operator_error = contract
        .set_maci_operator(&mut app, user3(), operator())
        .unwrap_err();
    assert_eq!(
        ContractError::Unauthorized {},
        not_validator_set_operator_error.downcast().unwrap()
    );

    _ = contract.set_maci_operator(&mut app, user2(), operator2());
    let user2_operator_addr = contract.get_validator_operator(&app, user2()).unwrap();
    assert_eq!(operator2(), user2_operator_addr);
    _ = contract.set_maci_operator_pubkey(&mut app, operator2(), pubkey2());
    let user2_operator_pubkey = contract.get_operator_pubkey(&app, operator2()).unwrap();
    assert_eq!(pubkey2(), user2_operator_pubkey);
    _ = contract.set_maci_operator_pubkey(&mut app, operator2(), pubkey3());
    let user2_operator_pubkey3 = contract.get_operator_pubkey(&app, operator2()).unwrap();
    assert_eq!(pubkey3(), user2_operator_pubkey3);

    _ = contract.set_validators_all(&mut app, owner());
    _ = contract.remove_validator(&mut app, owner(), user2());
    let validator_set_after_remove_user2 = contract.get_validators(&app).unwrap();
    assert_eq!(
        ValidatorSet {
            addresses: vec![user1(), user3()]
        },
        validator_set_after_remove_user2
    );

    let removed_validator_cannot_set_operator = contract
        .set_maci_operator(&mut app, user2(), operator3())
        .unwrap_err();
    assert_eq!(
        ContractError::Unauthorized {},
        removed_validator_cannot_set_operator.downcast().unwrap()
    );

    let cannot_set_same_operator_address = contract
        .set_maci_operator(&mut app, user3(), operator())
        .unwrap_err();
    assert_eq!(
        ContractError::ExistedMaciOperator {},
        cannot_set_same_operator_address.downcast().unwrap()
    );

    _ = contract.set_maci_operator(&mut app, user3(), operator3());
    let user3_operator_addr = contract.get_validator_operator(&app, user3()).unwrap();
    assert_eq!(operator3(), user3_operator_addr);

    let user3_register_with_user1_pubkey = contract
        .set_maci_operator_pubkey(&mut app, operator3(), pubkey1())
        .unwrap_err();
    assert_eq!(
        ContractError::PubkeyExisted {},
        user3_register_with_user1_pubkey.downcast().unwrap()
    );

    _ = contract.set_maci_operator_pubkey(&mut app, operator3(), pubkey3());
    let user3_operator_pubkey = contract.get_operator_pubkey(&app, operator3()).unwrap();
    assert_eq!(pubkey3(), user3_operator_pubkey);
}

#[test]
fn create_round_should_works() {
    let user1_coin_amount = 30u128;

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &user1(), coins(user1_coin_amount, DORA_DEMON))
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

    _ = contract.set_maci_operator_pubkey(&mut app, operator(), pubkey1());

    let user1_operator_pubkey = contract.get_operator_pubkey(&app, operator()).unwrap();
    assert_eq!(pubkey1(), user1_operator_pubkey);

    let create_round_with_wrong_circuit_type = contract
        .create_round(&mut app, user1(), operator(), 1u64, 0u64)
        .unwrap_err();
    assert_eq!(
        AmaciContractError::UnsupportedCircuitType {},
        create_round_with_wrong_circuit_type.downcast().unwrap()
    );

    let create_round_with_wrong_certification_system = contract
        .create_round(&mut app, user1(), operator(), 0u64, 1u64)
        .unwrap_err();
    assert_eq!(
        AmaciContractError::UnsupportedCertificationSystem {},
        create_round_with_wrong_certification_system.downcast().unwrap()
    );

    let resp = contract
        .create_round(&mut app, user1(), operator(), 0u64, 0u64)
        .unwrap();

    let amaci_contract_addr: InstantiationData = from_json(&resp.data.unwrap()).unwrap();
    println!("{:?}", amaci_contract_addr);
    let maci_contract = MaciContract::new(amaci_contract_addr.addr);
    let amaci_admin = maci_contract.query_admin(&app).unwrap();
    println!("{:?}", amaci_admin);
    assert_eq!(user1(), amaci_admin);

    let amaci_operator = maci_contract.query_operator(&app).unwrap();
    println!("{:?}", amaci_operator);
    assert_eq!(operator(), amaci_operator);

    let amaci_round_info = maci_contract.query_round_info(&app).unwrap();
    println!("{:?}", amaci_round_info);
}
