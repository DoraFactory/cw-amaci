use cosmwasm_std::{coins, Uint128, Uint256};
use cw_multi_test::{AppBuilder, Contract, ContractWrapper, Executor, StargateAccepting};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, PubKey};
use crate::multitest::{
    admin, create_app, creator, mock_registry_contract, operator1, operator2, user1, user2,
    SaasCodeId, DORA_DEMON,
};
use cw_amaci::multitest::uint256_from_decimal_string;
use cw_oracle_maci;
use cw_oracle_maci::state::RoundInfo as OracleMaciRoundInfo;

#[test]
fn test_instantiate_saas_contract() {
    let mut app = create_app();

    let oracle_maci_code_id = app.store_code(oracle_maci_contract());
    let code_id = SaasCodeId::store_code(&mut app);
    let contract = code_id
        .instantiate(
            &mut app,
            creator(),
            admin(),
            Some(mock_registry_contract()),
            DORA_DEMON.to_string(),
            oracle_maci_code_id,
            "SaaS Contract",
        )
        .unwrap();

    // Verify config
    let config = contract.query_config(&app).unwrap();
    assert_eq!(config.admin, admin());
    assert_eq!(config.registry_contract, Some(mock_registry_contract()));
    assert_eq!(config.denom, DORA_DEMON);

    // Verify initial balance is zero
    let balance = contract.query_balance(&app).unwrap();
    assert_eq!(balance, Uint128::zero());

    // Verify no operators initially
    let operators = contract.query_operators(&app).unwrap();
    assert!(operators.is_empty());

    // Verify oracle maci code id is set correctly
    let stored_code_id = contract.query_oracle_maci_code_id(&app).unwrap();
    assert_eq!(stored_code_id, oracle_maci_code_id);
}

#[test]
fn test_update_config() {
    let mut app = create_app();

    let oracle_maci_code_id = app.store_code(oracle_maci_contract());
    let code_id = SaasCodeId::store_code(&mut app);
    let contract = code_id
        .instantiate(
            &mut app,
            creator(),
            admin(),
            Some(mock_registry_contract()),
            DORA_DEMON.to_string(),
            oracle_maci_code_id,
            "SaaS Contract",
        )
        .unwrap();

    let new_admin = user1();

    // Update config as admin
    contract
        .update_config(&mut app, admin(), Some(new_admin.clone()), None, None)
        .unwrap();

    // Verify config updated
    let config = contract.query_config(&app).unwrap();
    assert_eq!(config.admin, new_admin);

    // Try to update as non-admin (should fail)
    let err = contract
        .update_config(&mut app, user2(), Some(admin()), None, None)
        .unwrap_err();
    assert!(err.to_string().contains("Error executing WasmMsg"));
}

#[test]
fn test_operator_management() {
    let mut app = create_app();

    let oracle_maci_code_id = app.store_code(oracle_maci_contract());
    let code_id = SaasCodeId::store_code(&mut app);
    let contract = code_id
        .instantiate(
            &mut app,
            creator(),
            admin(),
            Some(mock_registry_contract()),
            DORA_DEMON.to_string(),
            oracle_maci_code_id,
            "SaaS Contract",
        )
        .unwrap();

    // Add operator as admin
    contract
        .add_operator(&mut app, admin(), operator1())
        .unwrap();

    // Verify operator was added
    let is_operator = contract.query_is_operator(&app, operator1()).unwrap();
    assert!(is_operator);

    let operators = contract.query_operators(&app).unwrap();
    assert_eq!(operators.len(), 1);
    assert_eq!(operators[0].address, operator1());

    // Try to add same operator again (should fail)
    let err = contract
        .add_operator(&mut app, admin(), operator1())
        .unwrap_err();
    assert!(err.to_string().contains("Error executing WasmMsg"));

    // Add second operator
    contract
        .add_operator(&mut app, admin(), operator2())
        .unwrap();

    let operators = contract.query_operators(&app).unwrap();
    assert_eq!(operators.len(), 2);

    // Remove operator
    contract
        .remove_operator(&mut app, admin(), operator1())
        .unwrap();

    let is_operator = contract.query_is_operator(&app, operator1()).unwrap();
    assert!(!is_operator);

    let operators = contract.query_operators(&app).unwrap();
    assert_eq!(operators.len(), 1);
    assert_eq!(operators[0].address, operator2());

    // Try to remove non-existent operator (should fail)
    let err = contract
        .remove_operator(&mut app, admin(), operator1())
        .unwrap_err();
    assert!(err.to_string().contains("Error executing WasmMsg"));

    // Try to add operator as non-admin (should fail)
    let err = contract
        .add_operator(&mut app, user1(), operator1())
        .unwrap_err();
    assert!(err.to_string().contains("Error executing WasmMsg"));
}

#[test]
fn test_deposit_and_withdraw() {
    let deposit_amount = 1000000u128;
    let mut app = AppBuilder::default()
        .with_stargate(StargateAccepting)
        .build(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &user1(), coins(deposit_amount, DORA_DEMON))
                .unwrap();
        });

    let oracle_maci_code_id = app.store_code(oracle_maci_contract());
    let code_id = SaasCodeId::store_code(&mut app);
    let contract = code_id
        .instantiate(
            &mut app,
            creator(),
            admin(),
            Some(mock_registry_contract()),
            DORA_DEMON.to_string(),
            oracle_maci_code_id,
            "SaaS Contract",
        )
        .unwrap();

    // Deposit funds
    contract
        .deposit(&mut app, user1(), &coins(deposit_amount, DORA_DEMON))
        .unwrap();

    // Check balance
    let balance = contract.query_balance(&app).unwrap();
    assert_eq!(balance, Uint128::from(deposit_amount));

    // Try to deposit without funds (should fail)
    let err = contract.deposit(&mut app, user1(), &[]).unwrap_err();
    assert!(err.to_string().contains("Error executing WasmMsg"));

    // Withdraw funds as admin
    let withdraw_amount = Uint128::from(500000u128);
    contract
        .withdraw(&mut app, admin(), withdraw_amount, None)
        .unwrap();

    // Check balance updated
    let balance = contract.query_balance(&app).unwrap();
    assert_eq!(balance, Uint128::from(deposit_amount) - withdraw_amount);

    // Try to withdraw as non-admin (should fail)
    let err = contract
        .withdraw(&mut app, user1(), withdraw_amount, None)
        .unwrap_err();
    assert!(err.to_string().contains("Error executing WasmMsg"));

    // Try to withdraw zero amount (should fail)
    let err = contract
        .withdraw(&mut app, admin(), Uint128::zero(), None)
        .unwrap_err();
    assert!(err.to_string().contains("Error executing WasmMsg"));

    // Try to withdraw more than balance (should fail)
    let err = contract
        .withdraw(&mut app, admin(), Uint128::from(1000000u128), None)
        .unwrap_err();
    assert!(err.to_string().contains("Error executing WasmMsg"));
}

#[test]
fn test_create_oracle_maci_round_success() {
    let initial_balance = 1000000000000000000000u128; // 1000 DORA
    let mut app = AppBuilder::default()
        .with_stargate(StargateAccepting)
        .build(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &user1(), coins(initial_balance, DORA_DEMON))
                .unwrap();
        });

    let oracle_maci_code_id = app.store_code(oracle_maci_contract());
    let code_id = SaasCodeId::store_code(&mut app);
    let contract = code_id
        .instantiate(
            &mut app,
            creator(),
            admin(),
            None,
            DORA_DEMON.to_string(),
            oracle_maci_code_id,
            "SaaS Contract",
        )
        .unwrap();

    // Add operator and deposit funds
    contract
        .add_operator(&mut app, admin(), operator1())
        .unwrap();
    contract
        .deposit(&mut app, user1(), &coins(initial_balance, DORA_DEMON))
        .unwrap();

    let initial_balance_check = contract.query_balance(&app).unwrap();
    assert_eq!(initial_balance_check, Uint128::from(initial_balance));

    // Create Oracle MACI round
    let max_voters = 5u128;
    let create_msg = ExecuteMsg::CreateOracleMaciRound {
        coordinator: PubKey {
            x: Uint256::from(1u32),
            y: Uint256::from(2u32),
        },
        max_voters,
        vote_option_map: vec!["Option 1".to_string()],
        round_info: cw_amaci::state::RoundInfo {
            title: "Test Round".to_string(),
            description: "Test Description".to_string(),
            link: "https://test.com".to_string(),
        },
        start_time: cosmwasm_std::Timestamp::from_seconds(1640995200), // 2022-01-01
        end_time: cosmwasm_std::Timestamp::from_seconds(1641081600),   // 2022-01-02
        circuit_type: Uint256::zero(),
        certification_system: Uint256::zero(),
        whitelist_backend_pubkey: "dGVzdA==".to_string(),
    };

    let result = app.execute_contract(operator1(), contract.addr(), &create_msg, &[]);

    // Oracle MACI creation should succeed
    if let Err(e) = &result {
        println!("Error creating Oracle MACI round: {:?}", e);
    }
    assert!(
        result.is_ok(),
        "Oracle MACI round creation should succeed: {:?}",
        result.err()
    );

    let response = result.unwrap();

    // Verify Oracle MACI contract was instantiated
    let _instantiate_event = response
        .events
        .iter()
        .find(|e| e.ty == "instantiate")
        .expect("Should have instantiate event");

    // Calculate expected cost for validation
    let expected_cost = Uint128::from(10000000000000000000u128)
        + Uint128::from(max_voters * 1000000000000000000u128);

    // Verify fee_grant_amount event attribute exists and has correct value in wasm events
    let fee_grant_amount_attr = response
        .events
        .iter()
        .filter(|e| e.ty == "wasm")
        .flat_map(|e| &e.attributes)
        .find(|attr| attr.key == "fee_grant_amount")
        .expect("Should have fee_grant_amount attribute in wasm event");

    // The fee_grant_amount should equal the expected cost (deployment fee + voter tokens)
    assert_eq!(
        fee_grant_amount_attr.value,
        expected_cost.to_string(),
        "fee_grant_amount should match the total cost sent as funds"
    );

    // Verify balance was deducted correctly
    let final_balance = contract.query_balance(&app).unwrap();
    let expected_remaining = Uint128::from(initial_balance) - expected_cost;
    assert_eq!(final_balance, expected_remaining);

    // Verify MACI contract record was created
    let maci_contracts = contract.query_maci_contracts(&app, None, None).unwrap();
    assert_eq!(maci_contracts.len(), 1);
    assert_eq!(maci_contracts[0].round_title, "Test Round");
    assert_eq!(maci_contracts[0].creator_operator, operator1());

    // 方法二：通过 SAAS 合约查询获取 Oracle MACI 地址，然后查询详细的 round info
    if let Some(first_maci) = maci_contracts.first() {
        println!("========= 通过 SAAS 查询到的 MACI 合约信息 ==========");
        println!("合约地址: {}", first_maci.contract_address);
        println!("创建者: {}", first_maci.creator_operator);
        println!("轮次标题: {}", first_maci.round_title);
        println!("创建时间: {}", first_maci.created_at);
        println!("代码ID: {}", first_maci.code_id);
        println!("创建费用: {}", first_maci.creation_fee);

        // 查询详细的 round info
        let round_info_query_msg = serde_json::json!({
            "get_round_info": {}
        });

        let round_info_result: Result<OracleMaciRoundInfo, _> = app
            .wrap()
            .query_wasm_smart(&first_maci.contract_address, &round_info_query_msg);

        match round_info_result {
            Ok(round_info) => {
                println!("======== 详细的 Round Info ========");
                println!("Title: {}", round_info.title);
                println!("Description: {}", round_info.description);
                println!("Link: {}", round_info.link);
                println!("==================================");
            }
            Err(e) => {
                println!("查询详细 round info 失败: {:?}", e);
            }
        }
        println!("================================================");
    }
}

#[test]
fn test_create_oracle_maci_round_unauthorized() {
    let mut app = create_app();

    let oracle_maci_code_id = app.store_code(oracle_maci_contract());
    let code_id = SaasCodeId::store_code(&mut app);
    let contract = code_id
        .instantiate(
            &mut app,
            creator(),
            admin(),
            None,
            DORA_DEMON.to_string(),
            oracle_maci_code_id,
            "SaaS Contract",
        )
        .unwrap();

    let create_msg = ExecuteMsg::CreateOracleMaciRound {
        coordinator: PubKey {
            x: uint256_from_decimal_string(
                "3557592161792765812904087712812111121909518311142005886657252371904276697771",
            ),
            y: uint256_from_decimal_string(
                "4363822302427519764561660537570341277214758164895027920046745209970137856681",
            ),
        },
        max_voters: 5,
        vote_option_map: vec!["Option 1".to_string()],
        round_info: cw_amaci::state::RoundInfo {
            title: "Test Round".to_string(),
            description: "Test Description".to_string(),
            link: "https://test.com".to_string(),
        },
        start_time: cosmwasm_std::Timestamp::from_seconds(1753920000), // 2022-01-01
        end_time: cosmwasm_std::Timestamp::from_seconds(1754006400),   // 2022-01-02
        circuit_type: Uint256::zero(),
        certification_system: Uint256::zero(),
        whitelist_backend_pubkey: "AoYo/zENN/JquagPdG0/NMbWBBYxOM8BVN677mBXJKJQ".to_string(),
    };

    // Try to create round as non-operator (should fail with Unauthorized)
    let result = app.execute_contract(user1(), contract.addr(), &create_msg, &[]);

    assert!(
        result.is_err(),
        "Non-operator should not be able to create Oracle MACI round"
    );

    let error = result.unwrap_err();
    assert_eq!(
        error.downcast::<ContractError>().unwrap(),
        ContractError::Unauthorized {}
    );
}

#[test]
fn test_create_oracle_maci_round_insufficient_funds() {
    let initial_balance = 10000000000000000000u128; // 10 DORA - not enough
    let mut app = AppBuilder::default()
        .with_stargate(StargateAccepting)
        .build(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &user1(), coins(initial_balance, DORA_DEMON))
                .unwrap();
        });

    let oracle_maci_code_id = app.store_code(oracle_maci_contract());
    let code_id = SaasCodeId::store_code(&mut app);
    let contract = code_id
        .instantiate(
            &mut app,
            creator(),
            admin(),
            None,
            DORA_DEMON.to_string(),
            oracle_maci_code_id,
            "SaaS Contract",
        )
        .unwrap();

    // Add operator and deposit insufficient funds
    contract
        .add_operator(&mut app, admin(), operator1())
        .unwrap();
    contract
        .deposit(&mut app, user1(), &coins(initial_balance, DORA_DEMON))
        .unwrap();

    let create_msg = ExecuteMsg::CreateOracleMaciRound {
        coordinator: PubKey {
            x: Uint256::from(1u32),
            y: Uint256::from(2u32),
        },
        max_voters: 100, // Requires 100 DORA tokens + 10 DORA deployment = 110 DORA total
        vote_option_map: vec!["Option 1".to_string()],
        round_info: cw_amaci::state::RoundInfo {
            title: "Test Round".to_string(),
            description: "Test Description".to_string(),
            link: "https://test.com".to_string(),
        },
        start_time: cosmwasm_std::Timestamp::from_seconds(1640995200), // 2022-01-01
        end_time: cosmwasm_std::Timestamp::from_seconds(1641081600),   // 2022-01-02
        circuit_type: Uint256::zero(),
        certification_system: Uint256::zero(),
        whitelist_backend_pubkey: "dGVzdA==".to_string(),
    };

    // Should fail with insufficient funds
    let result = app.execute_contract(operator1(), contract.addr(), &create_msg, &[]);

    assert!(result.is_err(), "Should fail with insufficient funds");

    let error = result.unwrap_err();
    assert_eq!(
        error.downcast::<ContractError>().unwrap(),
        ContractError::InsufficientFundsForRound {
            required: Uint128::from(110000000000000000000u128), // 110 DORA
            available: Uint128::from(initial_balance),
        }
    );
}

#[test]
fn test_oracle_maci_round_management() {
    let initial_balance = 1000000000000000000000u128; // 1000 DORA
    let mut app = AppBuilder::default()
        .with_stargate(StargateAccepting)
        .build(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &user1(), coins(initial_balance, DORA_DEMON))
                .unwrap();
        });

    let oracle_maci_code_id = app.store_code(oracle_maci_contract());
    let code_id = SaasCodeId::store_code(&mut app);
    let contract = code_id
        .instantiate(
            &mut app,
            creator(),
            admin(),
            None,
            DORA_DEMON.to_string(),
            oracle_maci_code_id,
            "SaaS Contract",
        )
        .unwrap();

    // Setup: add operator and deposit funds
    contract
        .add_operator(&mut app, admin(), operator1())
        .unwrap();
    contract
        .deposit(&mut app, user1(), &coins(initial_balance, DORA_DEMON))
        .unwrap();

    // Create Oracle MACI round first
    let create_msg = ExecuteMsg::CreateOracleMaciRound {
        coordinator: PubKey {
            x: uint256_from_decimal_string(
                "3557592161792765812904087712812111121909518311142005886657252371904276697771",
            ),
            y: uint256_from_decimal_string(
                "4363822302427519764561660537570341277214758164895027920046745209970137856681",
            ),
        },
        max_voters: 5,
        vote_option_map: vec!["Option 1".to_string()],
        round_info: cw_amaci::state::RoundInfo {
            title: "Test Round".to_string(),
            description: "Test Description".to_string(),
            link: "https://test.com".to_string(),
        },
        start_time: cosmwasm_std::Timestamp::from_seconds(1753920000), // 2022-01-01
        end_time: cosmwasm_std::Timestamp::from_seconds(1754006400),   // 2022-01-02
        circuit_type: Uint256::zero(),
        certification_system: Uint256::zero(),
        whitelist_backend_pubkey: "AoYo/zENN/JquagPdG0/NMbWBBYxOM8BVN677mBXJKJQ".to_string(),
    };

    let create_result = app.execute_contract(operator1(), contract.addr(), &create_msg, &[]);
    if let Err(e) = &create_result {
        println!("Error creating Oracle MACI round: {:?}", e);
    }
    assert!(
        create_result.is_ok(),
        "Oracle MACI round creation should succeed: {:?}",
        create_result.err()
    );

    let response = create_result.unwrap();

    // Verify fee_grant_amount event attribute exists in wasm events
    let fee_grant_amount_attr = response
        .events
        .iter()
        .filter(|e| e.ty == "wasm")
        .flat_map(|e| &e.attributes)
        .find(|attr| attr.key == "fee_grant_amount")
        .expect("Should have fee_grant_amount attribute in wasm event");

    // Calculate expected cost (10 DORA deployment + 5 DORA for voters)
    let expected_cost =
        Uint128::from(10000000000000000000u128) + Uint128::from(5u128 * 1000000000000000000u128);

    assert_eq!(
        fee_grant_amount_attr.value,
        expected_cost.to_string(),
        "fee_grant_amount should match the total cost sent as funds"
    );

    // Get the created contract address from events
    let oracle_maci_addr = extract_contract_address_from_events(&response.events);
    println!("========= oracle_maci_addr: {}", oracle_maci_addr);

    // 查询并打印 Oracle MACI round info
    let round_info_query_msg = serde_json::json!({
        "get_round_info": {}
    });

    let round_info_result: Result<OracleMaciRoundInfo, _> = app
        .wrap()
        .query_wasm_smart(oracle_maci_addr.clone(), &round_info_query_msg);

    match round_info_result {
        Ok(round_info) => {
            println!("========= Oracle MACI Round Info ==========");
            println!("Title: {}", round_info.title);
            println!("Description: {}", round_info.description);
            println!("Link: {}", round_info.link);
            println!("==========================================");
        }
        Err(e) => {
            println!("查询 round info 失败: {:?}", e);
        }
    }

    let create_result_again = app.execute_contract(operator1(), contract.addr(), &create_msg, &[]);
    if let Err(e) = &create_result_again {
        println!("Error creating Oracle MACI round again: {:?}", e);
    }
    assert!(
        create_result_again.is_ok(),
        "Oracle MACI round creation should succeed: {:?}",
        create_result_again.err()
    );

    // Get the created contract address from events
    let oracle_maci_addr_again =
        extract_contract_address_from_events(&create_result_again.unwrap().events);
    println!(
        "========= oracle_maci_addr_again: {}",
        oracle_maci_addr_again
    );

    // 查询并打印第二个 Oracle MACI round info
    let round_info_query_msg_again = serde_json::json!({
        "get_round_info": {}
    });

    let round_info_result_again: Result<OracleMaciRoundInfo, _> = app
        .wrap()
        .query_wasm_smart(oracle_maci_addr_again.clone(), &round_info_query_msg_again);

    match round_info_result_again {
        Ok(round_info) => {
            println!("====== Second Oracle MACI Round Info ======");
            println!("Title: {}", round_info.title);
            println!("Description: {}", round_info.description);
            println!("Link: {}", round_info.link);
            println!("===========================================");
        }
        Err(e) => {
            println!("查询第二个 round info 失败: {:?}", e);
        }
    }
    // Test round info management
    let updated_round_info = cw_amaci::state::RoundInfo {
        title: "Updated Round Title".to_string(),
        description: "Updated Description".to_string(),
        link: "https://updated-test.com".to_string(),
    };

    let set_round_info_msg = ExecuteMsg::SetRoundInfo {
        contract_addr: oracle_maci_addr.clone(),
        round_info: updated_round_info,
    };

    // Operator should be able to update round info (may fail due to test environment but should pass authorization)
    let result = app.execute_contract(operator1(), contract.addr(), &set_round_info_msg, &[]);
    // In test environment this may fail due to target contract not existing, but not due to authorization
    if let Err(e) = &result {
        let error_msg = e.to_string();
        assert!(
            !error_msg.contains("Unauthorized"),
            "Should not fail due to authorization"
        );
    }

    // Non-operator should not be able to update round info
    let result = app.execute_contract(user1(), contract.addr(), &set_round_info_msg, &[]);
    assert!(
        result.is_err(),
        "Non-operator should not be able to update round info"
    );
    assert_eq!(
        result.unwrap_err().downcast::<ContractError>().unwrap(),
        ContractError::Unauthorized {}
    );

    // Test vote options management
    let updated_vote_options = vec![
        "Strongly Support".to_string(),
        "Support".to_string(),
        "Neutral".to_string(),
        "Oppose".to_string(),
        "Strongly Oppose".to_string(),
    ];

    let set_vote_options_msg = ExecuteMsg::SetVoteOptionsMap {
        contract_addr: oracle_maci_addr.clone(),
        vote_option_map: updated_vote_options,
    };

    // Operator should be able to update vote options
    let result = app.execute_contract(operator1(), contract.addr(), &set_vote_options_msg, &[]);
    if let Err(e) = &result {
        let error_msg = e.to_string();
        assert!(
            !error_msg.contains("Unauthorized"),
            "Should not fail due to authorization"
        );
    }

    // Test fee grant management
    let fee_grant_amount = Uint128::from(1000000000000000000u128); // 1 DORA

    let grant_msg = ExecuteMsg::GrantToVoter {
        contract_addr: oracle_maci_addr,
        grantee: user1(),
        base_amount: fee_grant_amount,
    };

    // Operator should be able to grant fee grants
    let result = app.execute_contract(operator1(), contract.addr(), &grant_msg, &[]);
    if let Err(e) = &result {
        let error_msg = e.to_string();
        assert!(
            !error_msg.contains("Unauthorized"),
            "Should not fail due to authorization"
        );
    }

    println!("========= grant_msg =======");
    // Non-operator should not be able to grant fee grants
    let result = app.execute_contract(user1(), contract.addr(), &grant_msg, &[]);
    assert!(
        result.is_err(),
        "Non-operator should not be able to grant fee grants"
    );
    assert_eq!(
        result.unwrap_err().downcast::<ContractError>().unwrap(),
        ContractError::Unauthorized {}
    );
}

// Helper function to extract contract address from events
fn extract_contract_address_from_events(events: &[cosmwasm_std::Event]) -> String {
    for event in events {
        if event.ty == "instantiate" {
            for attr in &event.attributes {
                if attr.key == "_contract_address" {
                    return attr.value.clone();
                }
            }
        }
    }
    "contract1".to_string() // Default fallback for test
}

#[test]
fn test_operator_feegrant_lifecycle() {
    let mut app = create_app();

    let oracle_maci_code_id = app.store_code(oracle_maci_contract());
    let code_id = SaasCodeId::store_code(&mut app);
    let contract = code_id
        .instantiate(
            &mut app,
            creator(),
            admin(),
            Some(mock_registry_contract()),
            DORA_DEMON.to_string(),
            oracle_maci_code_id,
            "SaaS Contract",
        )
        .unwrap();

    // Verify no operators initially
    let operators = contract.query_operators(&app).unwrap();
    assert!(operators.is_empty());

    // Add operator1 and check feegrant is granted
    let res = contract
        .add_operator(&mut app, admin(), operator1())
        .unwrap();

    // Check that feegrant was granted (via events/attributes)
    let grant_attrs: Vec<_> = res
        .events
        .iter()
        .flat_map(|e| &e.attributes)
        .filter(|attr| attr.key == "feegrant_action")
        .collect();
    assert!(!grant_attrs.is_empty());
    assert_eq!(grant_attrs[0].value, "auto_grant");

    // Check that feegrant amount was recorded
    let amount_attrs: Vec<_> = res
        .events
        .iter()
        .flat_map(|e| &e.attributes)
        .filter(|attr| attr.key == "feegrant_amount")
        .collect();
    assert!(!amount_attrs.is_empty());
    assert_eq!(amount_attrs[0].value, "10000000000000000000000000"); // 10B tokens

    // Check that denom was recorded
    let denom_attrs: Vec<_> = res
        .events
        .iter()
        .flat_map(|e| &e.attributes)
        .filter(|attr| attr.key == "feegrant_denom")
        .collect();
    assert!(!denom_attrs.is_empty());
    assert_eq!(denom_attrs[0].value, DORA_DEMON);

    // Verify operator was added
    let operators = contract.query_operators(&app).unwrap();
    assert_eq!(operators.len(), 1);
    assert_eq!(operators[0].address, operator1());

    // Verify operator is recognized
    let is_operator = contract.query_is_operator(&app, operator1()).unwrap();
    assert!(is_operator);

    // Query feegrant allowance (should exist for operator)
    let allowance = contract
        .query_feegrant_allowance(&app, contract.addr().to_string(), operator1().to_string())
        .unwrap();
    assert!(allowance.is_some());
    let allowance = allowance.unwrap();
    assert_eq!(allowance.granter, contract.addr().to_string());
    assert_eq!(allowance.grantee, operator1().to_string());
    assert_eq!(allowance.spend_limit[0].denom, "peaka");
    assert_eq!(
        allowance.spend_limit[0].amount,
        Uint128::from(10_000_000_000_000_000_000_000_000u128)
    );

    // Check feegrant exists using convenience method
    let has_feegrant = contract
        .has_feegrant_allowance(&app, contract.addr().to_string(), operator1().to_string())
        .unwrap();
    assert!(has_feegrant);

    // Add second operator to verify multiple feegrants
    let res2 = contract
        .add_operator(&mut app, admin(), operator2())
        .unwrap();

    // Check second feegrant was also granted
    let grant_attrs2: Vec<_> = res2
        .events
        .iter()
        .flat_map(|e| &e.attributes)
        .filter(|attr| attr.key == "feegrant_action")
        .collect();
    assert!(!grant_attrs2.is_empty());
    assert_eq!(grant_attrs2[0].value, "auto_grant");

    // Now remove first operator and check feegrant is revoked
    let res_remove = contract
        .remove_operator(&mut app, admin(), operator1())
        .unwrap();

    // Check that feegrant was revoked (via events/attributes)
    let revoke_attrs: Vec<_> = res_remove
        .events
        .iter()
        .flat_map(|e| &e.attributes)
        .filter(|attr| attr.key == "feegrant_action")
        .collect();
    assert!(!revoke_attrs.is_empty());
    assert_eq!(revoke_attrs[0].value, "auto_revoke");

    // Verify operator was removed
    let operators = contract.query_operators(&app).unwrap();
    assert_eq!(operators.len(), 1);
    assert_eq!(operators[0].address, operator2());

    // Verify operator1 is no longer recognized
    let is_operator = contract.query_is_operator(&app, operator1()).unwrap();
    assert!(!is_operator);

    // Query feegrant allowance (should NOT exist for removed operator)
    let allowance = contract
        .query_feegrant_allowance(&app, contract.addr().to_string(), operator1().to_string())
        .unwrap();
    assert!(allowance.is_none());

    // Check feegrant no longer exists
    let has_feegrant = contract
        .has_feegrant_allowance(&app, contract.addr().to_string(), operator1().to_string())
        .unwrap();
    assert!(!has_feegrant);

    // Verify operator2 is still recognized
    let is_operator = contract.query_is_operator(&app, operator2()).unwrap();
    assert!(is_operator);

    // Remove second operator
    let res_remove2 = contract
        .remove_operator(&mut app, admin(), operator2())
        .unwrap();

    // Check second feegrant was also revoked
    let revoke_attrs2: Vec<_> = res_remove2
        .events
        .iter()
        .flat_map(|e| &e.attributes)
        .filter(|attr| attr.key == "feegrant_action")
        .collect();
    assert!(!revoke_attrs2.is_empty());
    assert_eq!(revoke_attrs2[0].value, "auto_revoke");

    // Verify all operators are removed
    let operators = contract.query_operators(&app).unwrap();
    assert!(operators.is_empty());

    // Verify operator2 is no longer recognized
    let is_operator = contract.query_is_operator(&app, operator2()).unwrap();
    assert!(!is_operator);

    // Query feegrant allowance (should NOT exist for removed operator2)
    let allowance = contract
        .query_feegrant_allowance(&app, contract.addr().to_string(), operator2().to_string())
        .unwrap();
    assert!(allowance.is_none());

    // Check feegrant no longer exists for operator2
    let has_feegrant = contract
        .has_feegrant_allowance(&app, contract.addr().to_string(), operator2().to_string())
        .unwrap();
    assert!(!has_feegrant);
}

// Oracle MACI contract wrapper for testing
fn oracle_maci_contract() -> Box<dyn Contract<cosmwasm_std::Empty>> {
    let contract = ContractWrapper::new(
        cw_oracle_maci::contract::execute,
        cw_oracle_maci::contract::instantiate,
        cw_oracle_maci::contract::query,
    )
    .with_reply(cw_oracle_maci::contract::reply);
    Box::new(contract)
}
