#[cfg(test)]
mod test {
    use crate::error::ContractError;
    use crate::msg::{Groth16ProofType, PlonkProofType};
    use crate::multitest::{
        create_app, match_user_certificate, owner, uint256_from_decimal_string, user1,
        user1_certificate, user2, user2_certificate, user2_certificate_before, user3,
        user3_certificate_before, whitelist_ecosystem, whitelist_slope, whitelist_snapshot_height,
        whitelist_voting_power_mode, MaciCodeId,
    };
    use crate::state::{MessageData, Period, PeriodStatus, PubKey, RoundInfo};
    use cosmwasm_std::{Addr, Uint128, Uint256};
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

    #[test]
    fn instantiate_with_voting_time_isqv_should_works() {
        let msg_file_path = "./src/test/qv_test/msg.json";

        let mut msg_file = fs::File::open(msg_file_path).expect("Failed to open file");
        let mut msg_content = String::new();

        msg_file
            .read_to_string(&mut msg_content)
            .expect("Failed to read file");

        let data: MsgData = serde_json::from_str(&msg_content).expect("Failed to parse JSON");

        let result_file_path = "./src/test/qv_test/result.json";
        let mut result_file = fs::File::open(result_file_path).expect("Failed to open file");
        let mut result_content = String::new();
        result_file
            .read_to_string(&mut result_content)
            .expect("Failed to read file");

        let result_data: ResultData =
            serde_json::from_str(&result_content).expect("Failed to parse JSON");

        let pubkey_file_path = "./src/test/user_pubkey.json";

        let mut pubkey_file = fs::File::open(pubkey_file_path).expect("Failed to open file");
        let mut pubkey_content = String::new();

        pubkey_file
            .read_to_string(&mut pubkey_content)
            .expect("Failed to read file");
        let pubkey_data: UserPubkeyData =
            serde_json::from_str(&pubkey_content).expect("Failed to parse JSON");

        let mut app = create_app();
        let code_id = MaciCodeId::store_code(&mut app);
        let label = "Group";
        let contract = code_id
            .instantiate_with_voting_time_isqv(&mut app, owner(), label)
            .unwrap();

        // assert_eq!(
        //     ContractError::AlreadySetVotingTime {
        //         time_name: String::from("start_time")
        //     },
        //     start_voting_error.downcast().unwrap()
        // );

        let num_sign_up = contract.num_sign_up(&app).unwrap();
        assert_eq!(num_sign_up, Uint256::from_u128(0u128));

        let vote_option_map = contract.vote_option_map(&app).unwrap();
        let max_vote_options = contract.max_vote_options(&app).unwrap();
        assert_eq!(vote_option_map, vec!["1", "2", "3", "4", "5"]);
        assert_eq!(max_vote_options, Uint256::from_u128(5u128));
        _ = contract.set_vote_option_map(&mut app, owner());
        let new_vote_option_map = contract.vote_option_map(&app).unwrap();
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
        let sign_up_error = contract
            .sign_up(
                &mut app,
                Addr::unchecked(0.to_string()),
                test_pubkey.clone(),
                match_user_certificate(0).amount,
                match_user_certificate(0).certificate,
            )
            .unwrap_err();
        assert_eq!(
            ContractError::PeriodError {},
            sign_up_error.downcast().unwrap()
        ); // Cannot signup before voting phase

        _ = contract.set_vote_option_map(&mut app, owner());

        app.update_block(next_block); // Start Voting
                                      // let set_whitelist_only_in_pending = contract.set_whitelist(&mut app, owner()).unwrap_err();
                                      // assert_eq!(
                                      //     // Cannot register again after registration
                                      //     ContractError::PeriodError {},
                                      //     set_whitelist_only_in_pending.downcast().unwrap()
                                      // );
        let set_vote_option_map_error =
            contract.set_vote_option_map(&mut app, owner()).unwrap_err();
        assert_eq!(
            ContractError::PeriodError {},
            set_vote_option_map_error.downcast().unwrap()
        );

        let error_start_process_in_voting = contract.start_process(&mut app, owner()).unwrap_err();
        assert_eq!(
            ContractError::PeriodError {},
            error_start_process_in_voting.downcast().unwrap()
        );
        assert_eq!(
            Period {
                status: PeriodStatus::Pending
            },
            contract.get_period(&app).unwrap()
        );

        let test1_pubkey = PubKey {
            x: uint256_from_decimal_string(&pubkey_data.pubkeys[1][0]),
            y: uint256_from_decimal_string(&pubkey_data.pubkeys[1][1]),
        };
        let wrong_signature_sign_up = contract
            .sign_up(
                &mut app,
                Addr::unchecked('1'),
                test1_pubkey,
                match_user_certificate(0).amount,
                match_user_certificate(0).certificate,
            )
            .unwrap_err();
        assert_eq!(
            ContractError::InvalidSignature {},
            wrong_signature_sign_up.downcast().unwrap()
        );
        let not_whitelist = contract
            .query_is_whitelist(
                &app,
                "0".to_string(),
                match_user_certificate(1).amount,
                match_user_certificate(1).certificate,
            )
            .unwrap();
        assert_eq!(false, not_whitelist);
        let query_user_balance_before_sign_up = contract
            .query_white_balance_of(
                &app,
                "1".to_string(),
                user2_certificate_before().amount,
                user2_certificate_before().certificate,
            )
            .unwrap();
        assert_eq!(Uint256::from_u128(100), query_user_balance_before_sign_up);

        for i in 0..data.msgs.len() {
            if i < Uint256::from_u128(2u128).to_string().parse().unwrap() {
                let pubkey = PubKey {
                    x: uint256_from_decimal_string(&pubkey_data.pubkeys[i][0]),
                    y: uint256_from_decimal_string(&pubkey_data.pubkeys[i][1]),
                };
                let is_whitelist = contract
                    .query_is_whitelist(
                        &app,
                        i.to_string(),
                        match_user_certificate(i).amount,
                        match_user_certificate(i).certificate,
                    )
                    .unwrap();
                assert_eq!(true, is_whitelist);

                println!("---------- signup ---------- {:?}", i);
                let _ = contract.sign_up(
                    &mut app,
                    Addr::unchecked(i.to_string()),
                    pubkey,
                    match_user_certificate(i).amount,
                    match_user_certificate(i).certificate,
                );
            }

            let test3_pubkey = PubKey {
                x: uint256_from_decimal_string(&pubkey_data.pubkeys[1][0]),
                y: uint256_from_decimal_string(&pubkey_data.pubkeys[1][1]),
            };

            let user3_sign_up_with_zero_amount = contract
                .sign_up(
                    &mut app,
                    user3(),
                    test3_pubkey,
                    user3_certificate_before().amount,
                    user3_certificate_before().certificate,
                )
                .unwrap_err();

            assert_eq!(
                ContractError::AmountIsZero {},
                user3_sign_up_with_zero_amount.downcast().unwrap()
            );
            let message = MessageData {
                data: [
                    uint256_from_decimal_string(&data.msgs[i][0]),
                    uint256_from_decimal_string(&data.msgs[i][1]),
                    uint256_from_decimal_string(&data.msgs[i][2]),
                    uint256_from_decimal_string(&data.msgs[i][3]),
                    uint256_from_decimal_string(&data.msgs[i][4]),
                    uint256_from_decimal_string(&data.msgs[i][5]),
                    uint256_from_decimal_string(&data.msgs[i][6]),
                ],
            };

            let enc_pub = PubKey {
                x: uint256_from_decimal_string(&data.enc_pub_keys[i][0]),
                y: uint256_from_decimal_string(&data.enc_pub_keys[i][1]),
            };
            _ = contract.publish_message(&mut app, user2(), message, enc_pub);
        }

        let query_user_balance_after_sign_up = contract
            .query_white_balance_of(
                &app,
                "1".to_string(),
                match_user_certificate(1).amount,
                match_user_certificate(1).certificate,
            )
            .unwrap();
        assert_eq!(Uint256::from_u128(80), query_user_balance_after_sign_up);

        let sign_up_after_voting_end_error = contract
            .sign_up(
                &mut app,
                Addr::unchecked(0.to_string()),
                test_pubkey.clone(),
                match_user_certificate(0).amount,
                match_user_certificate(0).certificate,
            )
            .unwrap_err();
        assert_eq!(
            ContractError::AlreadySignedUp {},
            sign_up_after_voting_end_error.downcast().unwrap()
        );

        assert_eq!(
            contract.num_sign_up(&app).unwrap(),
            Uint256::from_u128(2u128)
        );

        assert_eq!(
            contract.msg_length(&app).unwrap(),
            Uint256::from_u128(3u128)
        );

        // Stop Voting Period
        app.update_block(next_block);

        let sign_up_after_voting_end_error = contract
            .sign_up(
                &mut app,
                Addr::unchecked(3.to_string()),
                test_pubkey.clone(),
                match_user_certificate(0).amount,
                match_user_certificate(0).certificate,
            )
            .unwrap_err();
        assert_eq!(
            // Cannot signup after voting phase ends
            ContractError::PeriodError {},
            sign_up_after_voting_end_error.downcast().unwrap()
        );

        // assert_eq!(
        //     ContractError::AlreadySetVotingTime {
        //         time_name: String::from("end_time")
        //     },
        //     stop_voting_error.downcast().unwrap()
        // );
        app.update_block(next_block);

        _ = contract.start_process(&mut app, owner());
        assert_eq!(
            Period {
                status: PeriodStatus::Processing
            },
            contract.get_period(&app).unwrap()
        );

        println!(
            "after start process: {:?}",
            contract.get_period(&app).unwrap()
        );

        let new_state_commitment = uint256_from_decimal_string(&data.new_state_commitment);
        let proof = Groth16ProofType {
                a: "1d357813049bc4b83ded0d9dab748251c70633d6283df4aef6c3c8f53da22942297e1f9820cdd8acd3719be1dc18c0d6d7d978b8022b10b2412c0be757d898cb".to_string(),
                b: "205d75e9165f8e472d935314381246d192e174262a19779afbb3fac8f9471b211b93759ce5a42fcb5c92a37b7013b9f9f72f13bd6d4190a7327d661b2a1530c205cc957a89cf5a4be26d822ea194bee53b59c8780f49e13968436a734c2e5de10f5fcf817e99122edce715d30bb63babbbdb7c541154c166ee2d9f42349957c8".to_string(),
                c: "15f91dba796a622d18dc73af0e50a5a7b2d9668f3cbd4015b4137b54c6743f5524080bdc6be18a94e8a3e638c684e4810465e065bb3c68d3c752e5fb8ea9ea65".to_string()
            };
        println!("process_message proof {:?}", proof);
        println!(
            "process_message new state commitment {:?}",
            new_state_commitment
        );
        _ = contract
            .process_message(&mut app, owner(), new_state_commitment, proof)
            .unwrap();

        _ = contract.stop_processing(&mut app, owner());
        println!(
            "after stop process: {:?}",
            contract.get_period(&app).unwrap()
        );

        let error_start_process_in_talling = contract.start_process(&mut app, owner()).unwrap_err();
        assert_eq!(
            ContractError::PeriodError {},
            error_start_process_in_talling.downcast().unwrap()
        );
        assert_eq!(
            Period {
                status: PeriodStatus::Tallying
            },
            contract.get_period(&app).unwrap()
        );
        let tally_path = "./src/test/qv_test/tally.json";
        let mut tally_file = fs::File::open(tally_path).expect("Failed to open file");
        let mut tally_content = String::new();
        tally_file
            .read_to_string(&mut tally_content)
            .expect("Failed to read file");

        let tally_data: TallyData =
            serde_json::from_str(&tally_content).expect("Failed to parse JSON");

        let new_tally_commitment = uint256_from_decimal_string(&tally_data.new_tally_commitment);

        let tally_proof = Groth16ProofType {
            a: "2274e1f6b71fc2887c4f746ff384f00fd9d2b4f8ed1d59853af2cb891058624a2e73d79f02de60ee49604e972e9dae72e5a3f3b63b7b0bb6167d1d7365f3af0b".to_string(),
            b: "147e97b696f2483f9be88419802de05a37c272328413907b1cadf61768e4abf604435ebd5462d1af60bee71de26d9a7259982f809f5edf3da7ecbb8c2b55dec40b403b2e4becd1587519488c8fcbf7e6b504dd68016e1ed48443ccced09d08c10a69014af748d7b2921449762eb7e870f0185dab186df6a5aeda4401e9a343cc".to_string(),
            c: "100005547853768af099c27f658c8b44d52bb94117a235243dfb243f3687395e2d3634cdce0cbe115d8d497e2330a907f965e4d9080183b381fb4ff30f98f02a".to_string()
        };

        _ = contract
            .process_tally(&mut app, owner(), new_tally_commitment, tally_proof)
            .unwrap();

        let results: Vec<Uint256> = result_data
            .results
            .iter()
            .map(|input| uint256_from_decimal_string(input))
            .collect();

        let salt = uint256_from_decimal_string(&tally_data.new_results_root_salt);
        _ = contract.stop_tallying(&mut app, owner(), results, salt);

        let all_result = contract.get_all_result(&app);
        println!("all_result: {:?}", all_result);
        let error_start_process = contract.start_process(&mut app, owner()).unwrap_err();
        assert_eq!(
            ContractError::PeriodError {},
            error_start_process.downcast().unwrap()
        );

        assert_eq!(
            Period {
                status: PeriodStatus::Ended
            },
            contract.get_period(&app).unwrap()
        );
    }

    // #[test]
    fn test_voting_power_calculation() {
        let slope = whitelist_slope();

        assert_eq!(slope, Uint256::from_u128(1000000u128));
        // 1 ATOM = 1 VC
        assert_eq!(
            Uint256::from_u128(1000000u128) / slope,
            Uint256::from_u128(1u128)
        );
        // 1.2 ATOM = 1 VC
        assert_eq!(
            Uint256::from_u128(1200000u128) / slope,
            Uint256::from_u128(1u128)
        );
        // 3 ATOM = 3 VC
        assert_eq!(
            Uint256::from_u128(3000000u128) / slope,
            Uint256::from_u128(3u128)
        );
        // 0.3 ATOM = 0 VC
        assert_eq!(
            Uint256::from_u128(300000u128) / slope,
            Uint256::from_u128(0u128)
        );
        // 0.9 ATOM = 0 VC
        assert_eq!(
            Uint256::from_u128(900000u128) / slope,
            Uint256::from_u128(0u128)
        );
        // 1.9 ATOM = 1 VC
        assert_eq!(
            Uint256::from_u128(1900000u128) / slope,
            Uint256::from_u128(1u128)
        );
        // 19000000000000000.099000 ATOM = 19000000000000000 VC
        assert_eq!(
            Uint256::from_u128(19000000000000000099000u128) / slope,
            Uint256::from_u128(19000000000000000u128)
        );
        assert_eq!(Uint256::from_u128(0u128) / slope, Uint256::from_u128(0u128));
    }

    #[test]
    fn instantiate_with_voting_time_isqv_with_no_signup_vote_should_works() {
        let msg_file_path = "./src/test/qv_test/msg.json";

        let mut msg_file = fs::File::open(msg_file_path).expect("Failed to open file");
        let mut msg_content = String::new();

        msg_file
            .read_to_string(&mut msg_content)
            .expect("Failed to read file");

        let data: MsgData = serde_json::from_str(&msg_content).expect("Failed to parse JSON");

        let result_file_path = "./src/test/qv_test/result.json";
        let mut result_file = fs::File::open(result_file_path).expect("Failed to open file");
        let mut result_content = String::new();
        result_file
            .read_to_string(&mut result_content)
            .expect("Failed to read file");

        let tally_path = "./src/test/qv_test/tally.json";
        let mut tally_file = fs::File::open(tally_path).expect("Failed to open file");
        let mut tally_content = String::new();
        tally_file
            .read_to_string(&mut tally_content)
            .expect("Failed to read file");

        let tally_data: TallyData =
            serde_json::from_str(&tally_content).expect("Failed to parse JSON");

        let pubkey_file_path = "./src/test/user_pubkey.json";

        let mut pubkey_file = fs::File::open(pubkey_file_path).expect("Failed to open file");
        let mut pubkey_content = String::new();

        pubkey_file
            .read_to_string(&mut pubkey_content)
            .expect("Failed to read file");

        let mut app = create_app();
        let code_id = MaciCodeId::store_code(&mut app);
        let label = "Group";

        let create_contract_with_wrong_circuit_type = code_id
            .instantiate_with_wrong_circuit_type(&mut app, owner(), label)
            .unwrap_err();
        assert_eq!(
            ContractError::UnsupportedCircuitType {},
            create_contract_with_wrong_circuit_type.downcast().unwrap()
        );

        let contract = code_id
            .instantiate_with_voting_time_isqv(&mut app, owner(), label)
            .unwrap();

        // assert_eq!(
        //     ContractError::AlreadySetVotingTime {
        //         time_name: String::from("start_time")
        //     },
        //     start_voting_error.downcast().unwrap()
        // );

        let num_sign_up = contract.num_sign_up(&app).unwrap();
        assert_eq!(num_sign_up, Uint256::from_u128(0u128));

        let vote_option_map = contract.vote_option_map(&app).unwrap();
        let max_vote_options = contract.max_vote_options(&app).unwrap();
        assert_eq!(vote_option_map, vec!["1", "2", "3", "4", "5"]);
        assert_eq!(max_vote_options, Uint256::from_u128(5u128));
        _ = contract.set_vote_option_map(&mut app, owner());
        let new_vote_option_map = contract.vote_option_map(&app).unwrap();
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
        let sign_up_error = contract
            .sign_up(
                &mut app,
                Addr::unchecked(0.to_string()),
                test_pubkey.clone(),
                match_user_certificate(0).amount,
                match_user_certificate(0).certificate,
            )
            .unwrap_err();
        assert_eq!(
            ContractError::PeriodError {},
            sign_up_error.downcast().unwrap()
        ); // Cannot signup before voting phase

        _ = contract.set_vote_option_map(&mut app, owner());

        app.update_block(next_block); // Start Voting
        let set_vote_option_map_error =
            contract.set_vote_option_map(&mut app, owner()).unwrap_err();
        assert_eq!(
            ContractError::PeriodError {},
            set_vote_option_map_error.downcast().unwrap()
        );

        let error_start_process_in_voting = contract.start_process(&mut app, owner()).unwrap_err();
        assert_eq!(
            ContractError::PeriodError {},
            error_start_process_in_voting.downcast().unwrap()
        );
        assert_eq!(
            Period {
                status: PeriodStatus::Pending
            },
            contract.get_period(&app).unwrap()
        );

        // Stop Voting Period
        app.update_block(next_block);
        let results = vec![
            Uint256::from_u128(0u128),
            Uint256::from_u128(0u128),
            Uint256::from_u128(0u128),
            Uint256::from_u128(0u128),
            Uint256::from_u128(0u128),
        ];
        let salt = uint256_from_decimal_string(&tally_data.new_results_root_salt);
        _ = contract.start_process(&mut app, owner());
        _ = contract.stop_processing(&mut app, owner());
        _ = contract.stop_tallying(&mut app, owner(), results, salt);
        let all_result = contract.get_all_result(&app);
        println!("all_result: {:?}", all_result);
        let end_period = contract.get_period(&app).unwrap();
        println!("end_period: {:?}", end_period);
        // let error_start_process = contract.start_process(&mut app, owner()).unwrap_err();
        // assert_eq!(
        //     ContractError::PeriodError {},
        //     error_start_process.downcast().unwrap()
        // );

        assert_eq!(
            Period {
                status: PeriodStatus::Ended
            },
            contract.get_period(&app).unwrap()
        );
    }
}
