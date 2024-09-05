use cosmwasm_std::{coins, from_binary, from_json, to_json_binary, Coin, Uint128};
use cw_multi_test::{next_block, App};

// use crate::error::ContractError;
// use crate::msg::ClaimsResponse;
use crate::{
    multitest::{
        contract_address, owner, uint256_from_decimal_string, user1, user2, user3,
        AmaciRegistryCodeId, DORA_DEMON, MIN_DEPOSIT_AMOUNT,
    },
    ContractError,
};
use cw_amaci::{
    msg::InstantiationData,
    multitest::{create_app, MaciCodeId, MaciContract},
    state::PubKey,
};
#[test]
fn instantiate_should_works() {
    let user1_coin_amount = 30u128;
    let user2_coin_amount = 20u128;
    let user3_coin_amount = 10u128;
    let min_deposit_coin_amount = 20u128;

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
    let contract = code_id.instantiate(&mut app, owner(), label).unwrap();

    // check winner
    let total = contract.get_total(&app).unwrap();
    assert_eq!(total, 0u128);
    app.update_block(next_block);

    let user1_pubkey = PubKey {
        x: uint256_from_decimal_string(
            "3557592161792765812904087712812111121909518311142005886657252371904276697771",
        ),
        y: uint256_from_decimal_string(
            "4363822302427519764561660537570341277214758164895027920046745209970137856681",
        ),
    };

    _ = contract.register(
        &mut app,
        user1(),
        user1_pubkey.clone(),
        // Uint128::from(bond_coin_amount),
        &coins(min_deposit_coin_amount, DORA_DEMON),
    );

    app.update_block(next_block);
    let total = contract.get_total(&app).unwrap();
    assert_eq!(total, min_deposit_coin_amount);

    let user1_register_again_after_registered = contract
        .register(
            &mut app,
            user1(),
            user1_pubkey,
            &coins(user1_coin_amount - min_deposit_coin_amount, DORA_DEMON),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::ExistedMaciOperator {},
        user1_register_again_after_registered.downcast().unwrap()
    );

    let user_1_balance = contract
        .balance_of(&app, user1().to_string(), DORA_DEMON.to_string())
        .unwrap();

    assert_eq!(
        user_1_balance,
        Coin {
            amount: Uint128::from(user1_coin_amount - min_deposit_coin_amount),
            denom: DORA_DEMON.to_string()
        }
    );

    let user3_pubkey = PubKey {
        x: uint256_from_decimal_string(
            "3557592161792765812904087712812111121909518311142005886657252371904276697771",
        ),
        y: uint256_from_decimal_string(
            "4363822302427519764561660537570341277214758164895027920046745209970137856681",
        ),
    };

    let user3_register_with_not_min_deposit_amount = contract
        .register(
            &mut app,
            user3(),
            user3_pubkey,
            &coins(user3_coin_amount, DORA_DEMON),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::InsufficientDeposit {
            min_deposit_amount: Uint128::from(MIN_DEPOSIT_AMOUNT)
        },
        user3_register_with_not_min_deposit_amount
            .downcast()
            .unwrap()
    );
    // _ = contract.upload_deactivate_message(
    //     &mut app,
    //     user1(),
    //     contract_address(),
    //     deactivate_message,
    // );
    // let deactivate_message_state = contract
    //     .get_deactivate_message(&app, contract_address())
    //     .unwrap();
    // assert_eq!(deactivate_message_state, deactivate_format_message);

    // let maci_operator = contract
    //     .get_maci_operator(&app, contract_address())
    //     .unwrap();
    // assert_eq!(maci_operator, user1());
}

#[test]
fn create_round_should_works() {
    let user1_coin_amount = 30u128;
    let min_deposit_coin_amount = 20u128;

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
        .instantiate(&mut app, owner(), label)
        .unwrap();

    let user1_pubkey = PubKey {
        x: uint256_from_decimal_string(
            "3557592161792765812904087712812111121909518311142005886657252371904276697771",
        ),
        y: uint256_from_decimal_string(
            "4363822302427519764561660537570341277214758164895027920046745209970137856681",
        ),
    };

    _ = contract.register(
        &mut app,
        user1(),
        user1_pubkey.clone(),
        // Uint128::from(bond_coin_amount),
        &coins(min_deposit_coin_amount, DORA_DEMON),
    );

    let resp = contract
        .create_round(&mut app, user1(), amaci_code_id.id(), user1())
        .unwrap();
    println!("{:?}", resp);

    let amaci_contract_addr: InstantiationData = from_json(&resp.data.unwrap()).unwrap();
    println!("{:?}", amaci_contract_addr);
    let maci_contract = MaciContract::new(amaci_contract_addr.addr);
    let amaci_admin = maci_contract.query_admin(&app).unwrap();
    println!("{:?}", amaci_admin);
    assert_eq!(user1(), amaci_admin);

    let amaci_operator = maci_contract.query_operator(&app).unwrap();
    println!("{:?}", amaci_operator);
    assert_eq!(user1(), amaci_operator);

    let amaci_round_info = maci_contract.query_round_info(&app).unwrap();
    println!("{:?}", amaci_round_info);
}
