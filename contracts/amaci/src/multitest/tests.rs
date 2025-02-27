#[cfg(test)]
mod test {
    use crate::error::ContractError;
    use crate::msg::Groth16ProofType;
    use crate::multitest::{
        create_app, owner, uint256_from_decimal_string, user1, user2, user3, MaciCodeId,
    };
    use crate::state::{
        DelayRecord, DelayRecords, DelayType, MessageData, Period, PeriodStatus, PubKey,
    };
    use cosmwasm_std::{coins, Addr, BlockInfo, Timestamp, Uint128, Uint256};
    use cw_multi_test::{next_block, AppBuilder, StargateAccepting};
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

    pub fn next_block_11_min(block: &mut BlockInfo) {
        block.time = block.time.plus_minutes(11);
        block.height += 1;
    }

    // #[test] TODO
    fn instantiate_with_voting_time_should_works() {
        let msg_file_path = "./src/test/msg_test.json";

        let mut msg_file = fs::File::open(msg_file_path).expect("Failed to open file");
        let mut msg_content = String::new();

        msg_file
            .read_to_string(&mut msg_content)
            .expect("Failed to read file");

        let data: MsgData = serde_json::from_str(&msg_content).expect("Failed to parse JSON");

        let result_file_path = "./src/test/result.json";
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
            .instantiate_with_voting_time(&mut app, owner(), user1(), user2(), label)
            .unwrap();

        // let start_voting_error = contract.start_voting(&mut app, owner()).unwrap_err();

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
        assert_eq!(vote_option_map, vec!["", "", "", "", ""]);
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
            )
            .unwrap_err();
        assert_eq!(
            ContractError::PeriodError {},
            sign_up_error.downcast().unwrap()
        ); // 不能在voting环节之前进行signup

        _ = contract.set_vote_option_map(&mut app, owner());

        app.update_block(next_block); // Start Voting
        let set_whitelist_only_in_pending = contract.set_whitelist(&mut app, owner()).unwrap_err();
        assert_eq!(
            // 注册之后不能再进行注册
            ContractError::PeriodError {},
            set_whitelist_only_in_pending.downcast().unwrap()
        );
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

        for i in 0..data.msgs.len() {
            if i < Uint256::from_u128(2u128).to_string().parse().unwrap() {
                let pubkey = PubKey {
                    x: uint256_from_decimal_string(&pubkey_data.pubkeys[i][0]),
                    y: uint256_from_decimal_string(&pubkey_data.pubkeys[i][1]),
                };

                println!("---------- signup ---------- {:?}", i);
                let _ = contract.sign_up(&mut app, Addr::unchecked(i.to_string()), pubkey);
            }
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

        // let sign_up_after_voting_end_error = contract
        //     .sign_up(
        //         &mut app,
        //         Addr::unchecked(0.to_string()),
        //         test_pubkey.clone(),
        //     )
        //     .unwrap_err();
        // assert_eq!(
        //     // 注册之后不能再进行注册
        //     ContractError::Unauthorized {},
        //     sign_up_after_voting_end_error.downcast().unwrap()
        // );

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
            )
            .unwrap_err();
        assert_eq!(
            // 不能投票环节结束之后不能进行sign up
            ContractError::PeriodError {},
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
            a: "27fb48285bc59bc74c9197857856cf5f3dcce55f22b83589e399240b8469e45725c5495e3ebcdd3bc04620fd13fed113c31d19a685f7f037daf02dde02d26e4f".to_string(),
            b: "0d1bd72809defb6e85ea48de4c28e9ec9dcd2bc5111acdb66b5cdb38ccf6d4e32bdeac48a806c2fd6cef8e09bfde1983961693c8d4a513777ba26b07f2abacba1efb7600f04e786d93f321c6df732eb0043548cfe12fa8a5aea848a500ef5b9728dbc747fc76993c16dadf2c8ef68f3d757afa6d4caf9a767c424ec0d7ff4932".to_string(),
            c: "2062c6bee5dad15af1ebcb0e623b27f7d29775774cc92b2a7554d1801af818940309fa215204181d3a1fef15d162aa779b8900e2b84d8b8fa22a20b65652eb46".to_string()
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
        let tally_path = "./src/test/tally_test.json";
        let mut tally_file = fs::File::open(tally_path).expect("Failed to open file");
        let mut tally_content = String::new();
        tally_file
            .read_to_string(&mut tally_content)
            .expect("Failed to read file");

        let tally_data: TallyData =
            serde_json::from_str(&tally_content).expect("Failed to parse JSON");

        let new_tally_commitment = uint256_from_decimal_string(&tally_data.new_tally_commitment);

        let tally_proof = Groth16ProofType {
            a: "2554bb7be658b5261bbcacef022d86dc55360f936a1473aa5c70c5b20083d7370deb7df6a8d0e74ae7f8b310725f3063407679fd99d23a7ad77b7d1bff5572d5".to_string(),
            b: "0fa4de46a0fc9d269314bbac4fb8f3425780bcde9b613a5252400216dadc3b5809f1d59c5f84892444c89712ab087cd708dcec5b77c108d9db73a8821be6720302f4820fec3af0e29b8a8aaf83db039d46703795d6275f934a14e8edc040e18f2dab2b05decd1b5bdb18631b9a8106714ceb5cf9fa6f4a4325cf4289a4025fc7".to_string(),
            c: "0d6a9f2eb8cfb28368bf6976f2925a3fb8ac0ead8dc95fc9a79318d0518f24801dced0525cbb2f15f24198bfe3f77c1065120be9dcbc3d10c77ca5861c410910".to_string()
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

    // #[test] TODO
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
            .instantiate_with_voting_time_isqv(&mut app, owner(), user1(), user2(), label)
            .unwrap();

        // let start_voting_error = contract.start_voting(&mut app, owner()).unwrap_err();

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
        assert_eq!(vote_option_map, vec!["", "", "", "", ""]);
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
            )
            .unwrap_err();
        assert_eq!(
            ContractError::PeriodError {},
            sign_up_error.downcast().unwrap()
        ); // 不能在voting环节之前进行signup

        _ = contract.set_vote_option_map(&mut app, owner());

        app.update_block(next_block); // Start Voting
        let set_whitelist_only_in_pending = contract.set_whitelist(&mut app, owner()).unwrap_err();
        assert_eq!(
            // 注册之后不能再进行注册
            ContractError::PeriodError {},
            set_whitelist_only_in_pending.downcast().unwrap()
        );
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

        for i in 0..data.msgs.len() {
            if i < Uint256::from_u128(2u128).to_string().parse().unwrap() {
                let pubkey = PubKey {
                    x: uint256_from_decimal_string(&pubkey_data.pubkeys[i][0]),
                    y: uint256_from_decimal_string(&pubkey_data.pubkeys[i][1]),
                };

                println!("---------- signup ---------- {:?}", i);
                let _ = contract.sign_up(&mut app, Addr::unchecked(i.to_string()), pubkey);
            }
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

        // let sign_up_after_voting_end_error = contract
        //     .sign_up(
        //         &mut app,
        //         Addr::unchecked(0.to_string()),
        //         test_pubkey.clone(),
        //     )
        //     .unwrap_err();
        // assert_eq!(
        //     // 注册之后不能再进行注册
        //     ContractError::Unauthorized {},
        //     sign_up_after_voting_end_error.downcast().unwrap()
        // );

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
            )
            .unwrap_err();
        assert_eq!(
            // 不能投票环节结束之后不能进行sign up
            ContractError::PeriodError {},
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
                a: "25b5c63b4d2f7d3ac4a01258040ea6ab731797144ec246c3af3c6578986b10720522540f38cab117c83e58f6540a43c7dd77c807ed436b344f9a137d8a4c8b32".to_string(),
                b: "01aba8a6b76bb1c7b301c2f0c15005a0550a94b68c0f19b01ff385e4c441f5a610ad81a1689db632c16c2054fd862cd1ad132a3b46926dd21769ff9e691c2a670ef6e81de05b039fd805422437e890581edd4db80469deefb2edcddcf2872dec15a7b27a5ea2c2886d04e5454b9d24918a90bf0865326217d0e8f78abdef18fb".to_string(),
                c: "02a00a70680f2e20f28521bdf8bd139cd2227051bcdf2d5744e85c2b3c5f2f642aceac09e1cc3fe487f587f4a6fa362d71ac6669f6870a0ed33a89a4c8c297e0".to_string()
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
            a: "2887519d960001d9a47a6338fadaa9ae57a52ed7ebd8a56c80616e4245762caf221b1a4188c4a6e8db5f968a6c04c56a4ca1b2f46a254f7b2737e444394e6f96".to_string(),
            b: "2dacd0fc846bf705ae591121f8fcd6f240dbd8eac23902c0da6fa791cf4a553c1f320f588c5ace3c42edcaeeb6242491accc6dde284d18d107952600b2dc91160687d1a8ff86fc397f0c19f3fd2f68d1a629a8a30f9d696561c70b342df1b97e20f79261ae47d812805ecaac01b6408cd5049383953439b97b58f1348831ac4e".to_string(),
            c: "09e8a2dcf849d84d05d567c482ab144e252755e820cb331eafab44ed96e13b28158341fa2103ac8efdebe336beed5ddec420ca0e3f6736aa7f7937418c0c4f29".to_string()
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

    // #[test] TODO
    fn instantiate_with_voting_time_1p1v_amaci_pre_add_key_should_works() {
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

        let mut app = create_app();
        let code_id = MaciCodeId::store_code(&mut app);
        let label = "Group";
        let contract = code_id
            .instantiate_with_voting_time_isqv_amaci(
                &mut app,
                owner(),
                user1(),
                user2(),
                user3(),
                label,
            )
            .unwrap();

        // let start_voting_error = contract.start_voting(&mut app, owner()).unwrap_err();

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
        assert_eq!(vote_option_map, vec!["", "", "", "", ""]);
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
            )
            .unwrap_err();
        assert_eq!(
            ContractError::PeriodError {},
            sign_up_error.downcast().unwrap()
        ); // 不能在voting环节之前进行signup

        _ = contract.set_vote_option_map(&mut app, owner());

        app.update_block(next_block); // Start Voting
        let set_whitelist_only_in_pending = contract.set_whitelist(&mut app, owner()).unwrap_err();
        assert_eq!(
            // 注册之后不能再进行注册
            ContractError::PeriodError {},
            set_whitelist_only_in_pending.downcast().unwrap()
        );
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

        let pubkey0 = PubKey {
            x: uint256_from_decimal_string(&pubkey_data.pubkeys[0][0]),
            y: uint256_from_decimal_string(&pubkey_data.pubkeys[0][1]),
        };

        let pubkey1 = PubKey {
            x: uint256_from_decimal_string(&pubkey_data.pubkeys[1][0]),
            y: uint256_from_decimal_string(&pubkey_data.pubkeys[1][1]),
        };

        let _ = contract.sign_up(&mut app, Addr::unchecked("0"), pubkey0);
        let _ = contract.sign_up(&mut app, Addr::unchecked("1"), pubkey1);

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
                    _ = contract.publish_deactivate_message(&mut app, user2(), message, enc_pub);
                }
                "proofDeactivate" => {
                    let data: ProofDeactivateData = deserialize_data(&entry.data);

                    assert_eq!(
                        contract.num_sign_up(&app).unwrap(),
                        Uint256::from_u128(2u128)
                    );

                    assert_eq!(
                        contract.dmsg_length(&app).unwrap(),
                        Uint256::from_u128(2u128)
                    );

                    let size = uint256_from_decimal_string(&data.size);
                    let new_deactivate_commitment =
                        uint256_from_decimal_string(&data.new_deactivate_commitment);
                    let new_deactivate_root =
                        uint256_from_decimal_string(&data.new_deactivate_root);
                    let proof = Groth16ProofType {
                                    a: "07eb1d9b0b358b2e4fe5e051bfd67aa3e57e2ab2f64f10e35d396ffd250b43e50433ae33cf1f829a23b7f326d8d2e4ff947c6f9778b788cf98336a6596ca2d16".to_string(),
                                    b: "0178e65e73c8e868900a5b439ac9c9f4c5dd7b1648b1f62bd5515a570fbf35a910fe35a737af956348436c2c62f046a08f35c0c7249bdaee25821122d1e3e11805f57494d28352120e88d1f75f560b3f15bea5af48d07e942df098b3e1aa95ff0a2541ae1aec50d71f30d01be5cd3d8a9d86ead1f190fb7d4c723bdcf9b11a51".to_string(),
                                    c: "1e146ab4c5b7388f8207d8e00c8d44d63786eb9a2deb07674b9e47ecb263541b22109d09c11658954333b6e62dacca8a72c088ddd8ab633765bc46bf88e97cd8".to_string()
                                };
                    println!("process_deactivate_message proof {:?}", proof);
                    println!(
                        "process_deactivate_message new state commitment {:?}",
                        new_deactivate_commitment
                    );
                    _ = contract
                        .process_deactivate_message(
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
                                    a: "053eb9bf62de01898e5d7049bfeaee4611b78b54f516ff4b0fd93ffcdc491d8b170e2c3de370f8eeec93ebb57e49279adc68fb137f4aafe1b4206d7186592673".to_string(),
                                    b: "2746ba15cb4478a1a90bd512844cd0e57070357ff17ad90964b699f962f4f24817ce4dcc89d350df5d63ae7f05f0069272c3d352cb92237e682222e68d52da0f00551f58de3a3cac33d6af2fb052e4ff4d42008b5f33b310756a5e7017919087284dc00b9753a3891872ee599467348976ec2d72703d46949a9b8093a97718eb".to_string(),
                                    c: "1832b7d8607c041bd1437f43fe1d207ad64bea58f346cc91d0c72d9c02bbc4031decf433ecafc3874f4bcedbfae591caaf87834ad6867c7d342b96b6299ddd0a".to_string()
                                };

                    println!("add_new_key proof {:?}", proof);
                    _ = contract
                        .pre_add_key(&mut app, owner(), new_key_pub, nullifier, d, proof)
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
                    _ = contract.publish_message(&mut app, user2(), message, enc_pub);
                }
                "processMessage" => {
                    let data: ProcessMessageData = deserialize_data(&entry.data);
                    app.update_block(next_block);

                    let sign_up_after_voting_end_error = contract
                        .sign_up(
                            &mut app,
                            Addr::unchecked(3.to_string()),
                            test_pubkey.clone(),
                        )
                        .unwrap_err();
                    assert_eq!(
                        // 不能投票环节结束之后不能进行sign up
                        ContractError::PeriodError {},
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

                    let error_stop_processing_with_not_finish_process =
                        contract.stop_processing(&mut app, owner()).unwrap_err();
                    assert_eq!(
                        ContractError::MsgLeftProcess {},
                        error_stop_processing_with_not_finish_process
                            .downcast()
                            .unwrap()
                    );

                    let new_state_commitment =
                        uint256_from_decimal_string(&data.new_state_commitment);
                    let proof = Groth16ProofType {
                            a: "1064da3b6dc28c0c1cf5be19ae0d7e653cd6b4fd7fad60fbdf388358e3238a5106cdf7446c0e37a5421ffc98ca27e2ad7c39cbce6bd0828293a18903fb488b11".to_string(),
                            b: "269766a5e7a27980fa446543f84984ce60f8998f3518f74dff73d1b044323d4f22df42cb66facc4ce30d4e1937abe342cf8fda8d10134a4c21d60ab8ffabcc7029fcf2f5f4870f4d54d807cbd8cde9e4a2c2bc8740d6c63d835045145f1851470c8ba81d9639c83ecbecf5a4495238b4fcc7f8317388422c049dd7874b265b4b".to_string(),
                            c: "13e4c1882e33e250de25c916d469ef2fe99e2dfd2a89e2c2369ba348903d7bd40cd1b811de0b35c2b2ece3ac156e12cb1e1114819fbd37a670d0f588f4f30bab".to_string()
                        };
                    println!("process_message proof {:?}", proof);
                    println!(
                        "process_message new state commitment {:?}",
                        new_state_commitment
                    );
                    println!("------ processMessage ------");
                    _ = contract
                        .process_message(&mut app, owner(), new_state_commitment, proof)
                        .unwrap();
                }
                "processTally" => {
                    let data: ProcessTallyData = deserialize_data(&entry.data);

                    _ = contract.stop_processing(&mut app, owner());
                    println!(
                        "after stop process: {:?}",
                        contract.get_period(&app).unwrap()
                    );

                    let error_start_process_in_talling =
                        contract.start_process(&mut app, owner()).unwrap_err();
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

                    let new_tally_commitment =
                        uint256_from_decimal_string(&data.new_tally_commitment);

                    let tally_proof = Groth16ProofType {
                                a: "2223e53e3b01380cc92390be785006738a510e3f371b0ab255a4adc5a77839410537bc9546e50d1b634b45c8607c59d3ff905a64de8de75ea3f43b6b77a569be".to_string(),
                                b: "1786ccb676689ce648bcb5c9afba636d3bfb15b14c5333802f1006f9338f869a12e033e0a68484c04b9c6f8c6ee01d23a3cc78b13b86ab5282f14961f01f0b8212a89a503e8f2e652c5f00fceca6e1033df0904bb8626a2d6515bd44488e40e4211d1a7f6996e41ee46f81a762af3132174aa4725334783a493a432d1828db80".to_string(),
                                c: "1e53064534ff278b93ba9c2df8a8d2accac3358f7486072a605990e38544cc292cde5cf0b444f3395b627edeabf892ef3020b2b90edc3936bcef2caa6d68dbcb".to_string()
                            };

                    _ = contract
                        .process_tally(&mut app, owner(), new_tally_commitment, tally_proof)
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
                    _ = contract.stop_tallying(&mut app, owner(), results, salt);

                    let all_result = contract.get_all_result(&app);
                    println!("all_result: {:?}", all_result);
                    let error_start_process =
                        contract.start_process(&mut app, owner()).unwrap_err();
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
                _ => println!("Unknown type: {}", entry.log_type),
            }
        }
    }

    // #[test] TODO
    fn instantiate_with_voting_time_and_test_grant_should_works() {
        let admin_coin_amount = 50u128;
        let bond_coin_amount = 10u128;
        const DORA_DEMON: &str = "peaka";

        let msg_file_path = "./src/test/msg_test.json";

        let mut msg_file = fs::File::open(msg_file_path).expect("Failed to open file");
        let mut msg_content = String::new();

        msg_file
            .read_to_string(&mut msg_content)
            .expect("Failed to read file");

        let data: MsgData = serde_json::from_str(&msg_content).expect("Failed to parse JSON");

        let mut app = AppBuilder::default()
            .with_stargate(StargateAccepting)
            .build(|router, _api, storage| {
                router
                    .bank
                    .init_balance(storage, &owner(), coins(admin_coin_amount, DORA_DEMON))
                    .unwrap();
            });

        let code_id = MaciCodeId::store_code(&mut app);
        let label = "Group";
        let contract = code_id
            .instantiate_with_voting_time_and_no_whitelist(&mut app, owner(), label)
            .unwrap();

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
        _ = contract.set_whitelist(&mut app, owner());

        let error_grant_in_pending = contract
            .grant(&mut app, owner(), &coins(bond_coin_amount, DORA_DEMON))
            .unwrap_err();
        assert_eq!(
            ContractError::PeriodError {},
            error_grant_in_pending.downcast().unwrap()
        );

        _ = contract.set_vote_option_map(&mut app, owner());

        app.update_block(next_block); // Start Voting

        let a = contract.grant(&mut app, owner(), &coins(bond_coin_amount, DORA_DEMON));
        println!("grant res: {:?}", a);
        let feegrant_amount = contract.query_total_feegrant(&app).unwrap();
        assert_eq!(Uint128::from(10000000000000u128), feegrant_amount);

        for i in 0..data.msgs.len() {
            if i < Uint256::from_u128(2u128).to_string().parse().unwrap() {
                let pubkey = PubKey {
                    x: uint256_from_decimal_string(&data.current_state_leaves[i][0]),
                    y: uint256_from_decimal_string(&data.current_state_leaves[i][1]),
                };

                println!("---------- signup ---------- {:?}", i);
                let _ = contract.sign_up(&mut app, Addr::unchecked(i.to_string()), pubkey);
            }
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
    }

    #[test]
    fn instantiate_with_voting_time_qv_amaci_should_works() {
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

        let mut app = create_app();
        let code_id = MaciCodeId::store_code(&mut app);
        let label = "Group";
        let contract = code_id
            .instantiate_with_voting_time_isqv_amaci(
                &mut app,
                owner(),
                user1(),
                user2(),
                user3(),
                label,
            )
            .unwrap();

        // let start_voting_error = contract.start_voting(&mut app, owner()).unwrap_err();

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
        assert_eq!(vote_option_map, vec!["", "", "", "", ""]);
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
            )
            .unwrap_err();
        assert_eq!(
            ContractError::PeriodError {},
            sign_up_error.downcast().unwrap()
        ); // 不能在voting环节之前进行signup

        _ = contract.set_vote_option_map(&mut app, owner());

        app.update_block(next_block); // Start Voting
        let set_whitelist_only_in_pending = contract.set_whitelist(&mut app, owner()).unwrap_err();
        assert_eq!(
            // 注册之后不能再进行注册
            ContractError::PeriodError {},
            set_whitelist_only_in_pending.downcast().unwrap()
        );
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

        let pubkey0 = PubKey {
            x: uint256_from_decimal_string(&pubkey_data.pubkeys[0][0]),
            y: uint256_from_decimal_string(&pubkey_data.pubkeys[0][1]),
        };

        let pubkey1 = PubKey {
            x: uint256_from_decimal_string(&pubkey_data.pubkeys[1][0]),
            y: uint256_from_decimal_string(&pubkey_data.pubkeys[1][1]),
        };

        let _ = contract.sign_up(&mut app, Addr::unchecked("0"), pubkey0.clone());

        let can_sign_up_error = contract
            .sign_up(&mut app, Addr::unchecked("0"), pubkey0.clone())
            .unwrap_err();
        assert_eq!(
            ContractError::UserAlreadyRegistered {},
            can_sign_up_error.downcast().unwrap()
        );

        let _ = contract.sign_up(&mut app, Addr::unchecked("1"), pubkey1.clone());

        assert_eq!(
            contract.num_sign_up(&app).unwrap(),
            Uint256::from_u128(2u128)
        );

        assert_eq!(
            contract.signuped(&app, pubkey0.x).unwrap(),
            Uint256::from_u128(1u128)
        );
        assert_eq!(
            contract.signuped(&app, pubkey1.x).unwrap(),
            Uint256::from_u128(2u128)
        );

        for entry in &logs_data {
            match entry.log_type.as_str() {
                // "setStateLeaf" => {
                //     let pubkey0 = PubKey {
                //         x: uint256_from_decimal_string(&pubkey_data.pubkeys[0][0]),
                //         y: uint256_from_decimal_string(&pubkey_data.pubkeys[0][1]),
                //     };
                // },
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
                    _ = contract.publish_deactivate_message(&mut app, user2(), message, enc_pub);
                }
                "proofDeactivate" => {
                    let data: ProofDeactivateData = deserialize_data(&entry.data);

                    assert_eq!(
                        contract.dmsg_length(&app).unwrap(),
                        Uint256::from_u128(2u128)
                    );

                    let size = uint256_from_decimal_string(&data.size);
                    let new_deactivate_commitment =
                        uint256_from_decimal_string(&data.new_deactivate_commitment);
                    let new_deactivate_root =
                        uint256_from_decimal_string(&data.new_deactivate_root);
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
                    _ = contract
                        .process_deactivate_message(
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
                    _ = contract
                        .add_key(&mut app, owner(), new_key_pub, nullifier, d, proof)
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
                    _ = contract.publish_message(&mut app, user2(), message, enc_pub);
                }
                "processMessage" => {
                    let data: ProcessMessageData = deserialize_data(&entry.data);
                    app.update_block(next_block_11_min);

                    let sign_up_after_voting_end_error = contract
                        .sign_up(
                            &mut app,
                            Addr::unchecked(3.to_string()),
                            test_pubkey.clone(),
                        )
                        .unwrap_err();
                    assert_eq!(
                        // 不能投票环节结束之后不能进行sign up
                        ContractError::PeriodError {},
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

                    let error_stop_processing_with_not_finish_process =
                        contract.stop_processing(&mut app, owner()).unwrap_err();
                    assert_eq!(
                        ContractError::MsgLeftProcess {},
                        error_stop_processing_with_not_finish_process
                            .downcast()
                            .unwrap()
                    );

                    let new_state_commitment =
                        uint256_from_decimal_string(&data.new_state_commitment);
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
                    _ = contract
                        .process_message(&mut app, owner(), new_state_commitment, proof)
                        .unwrap();
                }
                "processTally" => {
                    let data: ProcessTallyData = deserialize_data(&entry.data);

                    _ = contract.stop_processing(&mut app, owner());
                    println!(
                        "after stop process: {:?}",
                        contract.get_period(&app).unwrap()
                    );

                    let error_start_process_in_talling =
                        contract.start_process(&mut app, owner()).unwrap_err();
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

                    let new_tally_commitment =
                        uint256_from_decimal_string(&data.new_tally_commitment);

                    let tally_proof = Groth16ProofType {
                                a: "2a88fe840fa2eb49979acee8b545766fc83f28f128219041d3bf1e900fcf86a219b124cbf9b755a802186c391a154137eadf865a4b604452f5f98c4c533a1652".to_string(),
                                b: "0b6706904e637f888a9406db1529c84c26d068ad54bbfd3597de3e542f9230302cfdfdcd606e3544a63139b888fa561c2bf5ed928826d68c4f35e0fd07d491da27488896f67e261e8e3c6e33f947700b10eb6029daf6d9ae19add49e19fde2792563eec2f3fa6a43b1ec42c7d2f32b644c2f18e2b48d5dc552958b49c30f80c8".to_string(),
                                c: "0d4529ea7dd9c686c5673d48ee6a3fb3971a7cf12441f40e0ba6116046d64767288254a628cac0e46ccd3f1d1100ba2cd922d71066ae48b78283773b505665e0".to_string()
                            };

                    _ = contract
                        .process_tally(&mut app, owner(), new_tally_commitment, tally_proof)
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
                    app.update_block(next_block_11_min);
                    _ = contract.stop_tallying(&mut app, owner(), results, salt);

                    let all_result = contract.get_all_result(&app);
                    println!("all_result: {:?}", all_result);
                    let error_start_process =
                        contract.start_process(&mut app, owner()).unwrap_err();
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
                _ => println!("Unknown type: {}", entry.log_type),
            }
        }

        let delay_records = contract.query_delay_records(&app).unwrap();
        println!("delay_records: {:?}", delay_records);
        assert_eq!(
            delay_records,
            DelayRecords {
                records: vec![DelayRecord {
                    delay_timestamp: Timestamp::from_nanos(1571798084879000000),
                    delay_duration: 665,
                    delay_reason: String::from("Tallying has timed out after 665 seconds"),
                    delay_process_dmsg_count: Uint256::from_u128(0),
                    delay_type: DelayType::TallyDelay,
                }]
            }
        );
    }

    #[test]
    fn instantiate_with_wrong_voting_time_error() {
        let mut app = create_app();
        let code_id = MaciCodeId::store_code(&mut app);
        let label = "Group";
        let contract = code_id
            .instantiate_with_wrong_voting_time(&mut app, owner(), user1(), user2(), label)
            .unwrap_err();

        // let start_voting_error = contract.start_voting(&mut app, owner()).unwrap_err();

        assert_eq!(ContractError::WrongTimeSet {}, contract.downcast().unwrap());
    }

    #[test]
    fn test_amaci_process_deactivate_message_delay_data() {
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

        let mut app = create_app();
        let code_id = MaciCodeId::store_code(&mut app);
        let label = "Group";
        let contract = code_id
            .instantiate_with_voting_time_isqv_amaci(
                &mut app,
                owner(),
                user1(),
                user2(),
                user3(),
                label,
            )
            .unwrap();

        // let start_voting_error = contract.start_voting(&mut app, owner()).unwrap_err();

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
        assert_eq!(vote_option_map, vec!["", "", "", "", ""]);
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
            )
            .unwrap_err();
        assert_eq!(
            ContractError::PeriodError {},
            sign_up_error.downcast().unwrap()
        ); // 不能在voting环节之前进行signup

        _ = contract.set_vote_option_map(&mut app, owner());

        app.update_block(next_block); // Start Voting
        let set_whitelist_only_in_pending = contract.set_whitelist(&mut app, owner()).unwrap_err();
        assert_eq!(
            // 注册之后不能再进行注册
            ContractError::PeriodError {},
            set_whitelist_only_in_pending.downcast().unwrap()
        );
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

        let pubkey0 = PubKey {
            x: uint256_from_decimal_string(&pubkey_data.pubkeys[0][0]),
            y: uint256_from_decimal_string(&pubkey_data.pubkeys[0][1]),
        };

        let pubkey1 = PubKey {
            x: uint256_from_decimal_string(&pubkey_data.pubkeys[1][0]),
            y: uint256_from_decimal_string(&pubkey_data.pubkeys[1][1]),
        };

        let _ = contract.sign_up(&mut app, Addr::unchecked("0"), pubkey0.clone());

        let can_sign_up_error = contract
            .sign_up(&mut app, Addr::unchecked("0"), pubkey0.clone())
            .unwrap_err();
        assert_eq!(
            ContractError::UserAlreadyRegistered {},
            can_sign_up_error.downcast().unwrap()
        );

        let _ = contract.sign_up(&mut app, Addr::unchecked("1"), pubkey1.clone());

        assert_eq!(
            contract.num_sign_up(&app).unwrap(),
            Uint256::from_u128(2u128)
        );

        assert_eq!(
            contract.signuped(&app, pubkey0.x).unwrap(),
            Uint256::from_u128(1u128)
        );
        assert_eq!(
            contract.signuped(&app, pubkey1.x).unwrap(),
            Uint256::from_u128(2u128)
        );

        for entry in &logs_data {
            match entry.log_type.as_str() {
                "publishDeactivateMessage" => {
                    println!("publishDeactivateMessage =================");
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
                    _ = contract.publish_deactivate_message(&mut app, user2(), message, enc_pub);
                }
                "proofDeactivate" => {
                    let data: ProofDeactivateData = deserialize_data(&entry.data);

                    assert_eq!(
                        contract.dmsg_length(&app).unwrap(),
                        Uint256::from_u128(2u128)
                    );

                    let size = uint256_from_decimal_string(&data.size);
                    let new_deactivate_commitment =
                        uint256_from_decimal_string(&data.new_deactivate_commitment);
                    let new_deactivate_root =
                        uint256_from_decimal_string(&data.new_deactivate_root);
                    let proof = Groth16ProofType {
                                    a: "2fac29af2cad382c07952b42c10b282d6ee5c27032548c370fdf40c693965b98239bb54fb0546480075f7e93f7f46acdacfecf3eb40fb3c16f9b13287d15fd7a".to_string(),
                                    b: "18fb4503928bda6fc6aa377170b80fb3e2c73161c78c936bca222cb233318c7517ca194640de6b7790ec65ea7e46891089567d86a9fe8e419ad5e5d27e2cf96a2cf5383ef516ea8d14754c2e9e132fe566dd32eb23cd0de3543398a03a1c15f02a75014c4db8598d472112b292bbdde2968c409b759dbe76dec21da24b09d1a1".to_string(),
                                    c: "18f024873175339f2e939c8bc8a369daa56257564f3e23b0cf4b635e5721f0d1285e5d66fc1dd69f581a2b146083267e4ce9a3c21e46f488af2ed9289bd00714".to_string()
                                };
                    app.update_block(next_block_11_min);
                    _ = contract
                        .process_deactivate_message(
                            &mut app,
                            owner(),
                            size,
                            new_deactivate_commitment,
                            new_deactivate_root,
                            proof,
                        )
                        .unwrap();
                }
                _ => println!("Unknown type: {}", entry.log_type),
            }
        }

        let delay_records = contract.query_delay_records(&app).unwrap();
        println!("============================");
        println!("delay_records: {:?}", delay_records);
        assert_eq!(
            delay_records,
            DelayRecords {
                records: vec![DelayRecord {
                    delay_timestamp: Timestamp::from_nanos(1571797424879305533),
                    delay_duration: 660,
                    delay_reason: String::from(
                        "Processing of 2 deactivate messages has timed out after 660 seconds"
                    ),
                    delay_process_dmsg_count: Uint256::from_u128(2),
                    delay_type: DelayType::DeactivateDelay,
                }]
            }
        );
    }
}
