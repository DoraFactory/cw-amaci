#[cfg(test)]
mod test {
    use cosmwasm_std::{coins, Coin, Uint128};
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
    use cw_amaci::state::PubKey;

    // #[test]
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
        let deactivate_message = vec![
            vec![
                uint256_from_decimal_string(
                    "7747057536760136005430228262435826264866580124843536896813145526144814116982",
                ),
                uint256_from_decimal_string(
                    "18328267626578854848326897321493160357703899589757355464037146322948839521936",
                ),
                uint256_from_decimal_string(
                    "15302024921945581093264101479484122274672654005630938006953421086920203917576",
                ),
                uint256_from_decimal_string(
                    "16644390621180328819121471049917891389532203684839145910292539858102955405675",
                ),
                uint256_from_decimal_string(
                    "8418242452403936823096676468642419860420471132369414923867387559012728451588",
                ),
            ],
            vec![
                uint256_from_decimal_string(
                    "12464466727380559741327029120716347565653310312488492293821270525711683451322",
                ),
                uint256_from_decimal_string(
                    "13309763630590930088453867560680909228282105989053894048998918693101765779139",
                ),
                uint256_from_decimal_string(
                    "4484921303738698851059972318346660239747562407935541875738545702197977643459",
                ),
                uint256_from_decimal_string(
                    "11866219424993283184335358483746244768886471962890428914681952211991059471133",
                ),
                uint256_from_decimal_string(
                    "10251843967876693474360077990049981506696856835920530518366732065775811188590",
                ),
            ],
        ];

        let deactivate_format_message: Vec<Vec<String>> = deactivate_message
            .iter()
            .map(|input| input.iter().map(|f| f.to_string()).collect())
            .collect();
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

        let maci_operator = contract
            .get_maci_operator(&app, contract_address())
            .unwrap();
        assert_eq!(maci_operator, user1());
    }

    //     _ = contract.add_whitelist(&mut app, operator());
    //     let contract_balance_after_add_whitelist = contract.get_total(&app).unwrap();
    //     assert_eq!(
    //         contract_balance_after_add_whitelist,
    //         bond_coin_amount - GAS_AMOUNT * 2 // user1, user2 gas
    //     );

    //     let get_user1_claim_data = contract.get_claim(&app, user1()).unwrap();
    //     assert_eq!(
    //         get_user1_claim_data,
    //         ClaimsResponse {
    //             amount: Uint128::from(BASE_AMOUNT),
    //             denom: DORA_DEMON.to_string()
    //         }
    //     );

    //     _ = contract.claim(&mut app, user1());
    //     let get_user1_claim_data_after_claim = contract.get_claim(&app, user1()).unwrap();
    //     assert_eq!(
    //         get_user1_claim_data_after_claim,
    //         ClaimsResponse {
    //             amount: Uint128::from(0u128),
    //             denom: DORA_DEMON.to_string()
    //         }
    //     );

    //     let get_total_after_claim = contract.get_total(&app).unwrap();
    //     assert_eq!(
    //         get_total_after_claim,
    //         bond_coin_amount - BASE_AMOUNT - GAS_AMOUNT * 2 // 50 - 20 - 2 = 28
    //     );

    //     let user1_claim_after_claimed = contract.claim(&mut app, user1()).unwrap_err();
    //     assert_eq!(
    //         ContractError::AlreadyClaimed {},
    //         user1_claim_after_claimed.downcast().unwrap()
    //     );

    //     let user2_claim_is_not_enough = contract.claim(&mut app, user2()).unwrap_err();
    //     assert_eq!(
    //         ContractError::BondTokenNotEnough {},
    //         user2_claim_is_not_enough.downcast().unwrap()
    //     );

    //     let get_total_after_error_claim = contract.get_total(&app).unwrap();
    //     assert_eq!(
    //         get_total_after_error_claim,
    //         bond_coin_amount - BASE_AMOUNT - GAS_AMOUNT * 2 // 50 - 20 - 2 = 28
    //     );

    //     let admin_balance = contract
    //         .balance_of(&app, owner().to_string(), DORA_DEMON.to_string())
    //         .unwrap();
    //     assert_eq!(
    //         admin_balance,
    //         Coin {
    //             amount: Uint128::from(BASE_AMOUNT),
    //             denom: DORA_DEMON.to_string()
    //         }
    //     );

    //     let contract_balance = contract.get_total(&app).unwrap();
    //     assert_eq!(
    //         contract_balance,
    //         bond_coin_amount - BASE_AMOUNT - GAS_AMOUNT * 2 // 50 - 20 - 2 = 28
    //     );

    //     let user_1_balance = contract
    //         .balance_of(&app, user1().to_string(), DORA_DEMON.to_string())
    //         .unwrap();

    //     assert_eq!(
    //         user_1_balance,
    //         Coin {
    //             amount: Uint128::from(BASE_AMOUNT + GAS_AMOUNT),
    //             denom: DORA_DEMON.to_string()
    //         }
    //     );
    //     let user_2_balance = contract
    //         .balance_of(&app, user2().to_string(), DORA_DEMON.to_string())
    //         .unwrap();
    //     assert_eq!(
    //         user_2_balance,
    //         Coin {
    //             amount: Uint128::from(GAS_AMOUNT),
    //             denom: DORA_DEMON.to_string()
    //         }
    //     );
    //     _ = contract.add_whitelist(&mut app, owner());

    //     let get_user1_claim_data_after_claim = contract.get_claim(&app, user1()).unwrap();
    //     assert_eq!(
    //         get_user1_claim_data_after_claim,
    //         ClaimsResponse {
    //             amount: Uint128::from(0u128),
    //             denom: DORA_DEMON.to_string()
    //         }
    //     );

    //     let user_can_not_add_whitelist_error =
    //         contract.add_new_whitelist(&mut app, user1()).unwrap_err();
    //     assert_eq!(
    //         ContractError::Unauthorized {},
    //         user_can_not_add_whitelist_error.downcast().unwrap()
    //     );

    //     let user_can_not_withdraw_error = contract.withdraw(&mut app, user1(), None).unwrap_err();
    //     assert_eq!(
    //         ContractError::Unauthorized {},
    //         user_can_not_withdraw_error.downcast().unwrap()
    //     );

    //     let operator_can_not_withdraw_error =
    //         contract.withdraw(&mut app, operator(), None).unwrap_err();
    //     assert_eq!(
    //         ContractError::Unauthorized {},
    //         operator_can_not_withdraw_error.downcast().unwrap()
    //     );

    //     let new_amount = admin_coin_amount - bond_coin_amount;
    //     _ = contract.bond(&mut app, owner(), &coins(new_amount, DORA_DEMON));

    //     let contract_balance = contract.get_total(&app).unwrap();
    //     assert_eq!(
    //         contract_balance,
    //         bond_coin_amount - BASE_AMOUNT - GAS_AMOUNT * 2 + new_amount
    //     ); // 28u128
    //     let user1_in_whitelist = contract.in_whitelist(&app, user1()).unwrap();
    //     assert_eq!(user1_in_whitelist, true);

    //     let user3_not_in_whitelist = contract.in_whitelist(&app, user3()).unwrap();
    //     assert_eq!(user3_not_in_whitelist, false);

    //     _ = contract.add_new_whitelist(&mut app, owner());
    //     let get_user3_claim_data = contract.get_claim(&app, user3()).unwrap();
    //     assert_eq!(
    //         get_user3_claim_data,
    //         ClaimsResponse {
    //             amount: Uint128::from(BASE_AMOUNT),
    //             denom: DORA_DEMON.to_string()
    //         }
    //     );
    //     _ = contract.claim(&mut app, user3());

    //     let get_user3_claim_data_after_claim = contract.get_claim(&app, user3()).unwrap();
    //     assert_eq!(
    //         get_user3_claim_data_after_claim,
    //         ClaimsResponse {
    //             amount: Uint128::from(0u128),
    //             denom: DORA_DEMON.to_string()
    //         }
    //     );

    //     let contract_balance_after_user3_claim = contract.get_total(&app).unwrap();
    //     assert_eq!(
    //         contract_balance_after_user3_claim,
    //         bond_coin_amount - BASE_AMOUNT - GAS_AMOUNT * 2 + new_amount
    //             - BASE_AMOUNT
    //             - GAS_AMOUNT * 2 // 30 - 20 - 2 + 20 - 20 - 2 = 6
    //     );

    //     _ = contract.add_reward(&mut app, operator());

    //     let get_user3_claim_data = contract.get_claim(&app, user3()).unwrap();
    //     assert_eq!(
    //         get_user3_claim_data,
    //         ClaimsResponse {
    //             amount: Uint128::from(REWARD_AMOUNT),
    //             denom: DORA_DEMON.to_string()
    //         }
    //     );
    //     _ = contract.claim(&mut app, user3());

    //     let get_user3_claim_data = contract.get_claim(&app, user3()).unwrap();
    //     assert_eq!(
    //         get_user3_claim_data,
    //         ClaimsResponse {
    //             amount: Uint128::from(0u128),
    //             denom: DORA_DEMON.to_string()
    //         }
    //     );
    //     let contract_balance_after_user3_claim = contract.get_total(&app).unwrap();

    //     _ = contract.add_reward(&mut app, operator());

    //     let get_user3_claim_data_1 = contract.get_claim(&app, user3()).unwrap();
    //     assert_eq!(
    //         get_user3_claim_data_1,
    //         ClaimsResponse {
    //             amount: Uint128::from(0u128),
    //             denom: DORA_DEMON.to_string()
    //         }
    //     );

    //     assert_eq!(
    //         contract_balance_after_user3_claim,
    //         bond_coin_amount - BASE_AMOUNT - GAS_AMOUNT * 2 + new_amount
    //             - BASE_AMOUNT
    //             - GAS_AMOUNT * 2
    //             - REWARD_AMOUNT // 30 - 20 - 2 + 20 - 20 - 2 = 6
    //     );

    //     let claim_again_error = contract.claim(&mut app, user3()).unwrap_err();
    //     assert_eq!(
    //         ContractError::AlreadyClaimed {},
    //         claim_again_error.downcast().unwrap()
    //     );

    //     let not_admin_change_operator_error = contract
    //         .change_operator(&mut app, operator(), operator2())
    //         .unwrap_err();
    //     assert_eq!(
    //         ContractError::Unauthorized {},
    //         not_admin_change_operator_error.downcast().unwrap()
    //     );

    //     _ = contract.change_operator(&mut app, owner(), operator2());

    //     let operator_canot_execute_after_change_oper = contract
    //         .add_whitelist_final(&mut app, operator())
    //         .unwrap_err();

    //     assert_eq!(
    //         ContractError::Unauthorized {},
    //         operator_canot_execute_after_change_oper.downcast().unwrap()
    //     );

    //     let operator_canot_execute_after_change_oper = contract
    //         .add_whitelist_final(&mut app, operator())
    //         .unwrap_err();

    //     assert_eq!(
    //         ContractError::Unauthorized {},
    //         operator_canot_execute_after_change_oper.downcast().unwrap()
    //     );

    //     _ = contract.add_whitelist_final(&mut app, operator2());

    //     let contract_balance_after_operator2_add = contract.get_total(&app).unwrap();

    //     assert_eq!(
    //         contract_balance_after_operator2_add,
    //         bond_coin_amount - BASE_AMOUNT - GAS_AMOUNT * 2 + new_amount
    //             - BASE_AMOUNT
    //             - GAS_AMOUNT * 2
    //             - REWARD_AMOUNT
    //             - GAS_AMOUNT // 30 - 20 - 2 + 20 - 20 - 2 - 1 = 5
    //     );
    // }
}
