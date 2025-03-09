#[cfg(test)]
mod tests;

use anyhow::Result as AnyResult;

use crate::state::{
    DelayRecords, MaciParameters, MessageData, Period, PubKey, RoundInfo, VotingTime,
};
use crate::utils::uint256_from_hex_string;
use crate::{
    contract::{execute, instantiate, query},
    msg::*,
};

use cosmwasm_std::testing::{MockApi, MockStorage};
use cosmwasm_std::{Addr, Coin, Empty, StdResult, Timestamp, Uint128, Uint256};
use cw_multi_test::App as DefaultApp;
use cw_multi_test::{
    no_init, AppBuilder, AppResponse, BankKeeper, ContractWrapper, DistributionKeeper, Executor,
    FailingModule, GovFailingModule, IbcFailingModule, StakeKeeper, StargateAccepting, WasmKeeper,
};
use num_bigint::BigUint;

pub fn uint256_from_decimal_string(decimal_string: &str) -> Uint256 {
    assert!(
        decimal_string.len() <= 77,
        "the decimal length can't abrove 77"
    );

    let decimal_number = BigUint::parse_bytes(decimal_string.as_bytes(), 10)
        .expect("Failed to parse decimal string");

    let byte_array = decimal_number.to_bytes_be();

    let hex_string = hex::encode(byte_array);
    uint256_from_hex_string(&hex_string)
}
pub const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";
// pub const ARCH_DEMON: &str = "aconst";
// pub const ARCH_DECIMALS: u8 = 18;

pub type App<ExecC = Empty, QueryC = Empty> = cw_multi_test::App<
    BankKeeper,
    MockApi,
    MockStorage,
    FailingModule<ExecC, QueryC, Empty>,
    WasmKeeper<ExecC, QueryC>,
    StakeKeeper,
    DistributionKeeper,
    IbcFailingModule,
    GovFailingModule,
    StargateAccepting,
>;

pub fn create_app() -> App {
    AppBuilder::new()
        .with_stargate(StargateAccepting)
        .build(no_init)
}

#[derive(Clone, Debug, Copy)]
pub struct MaciCodeId(u64);

impl MaciCodeId {
    pub fn id(&self) -> u64 {
        self.0
    }

    pub fn store_default_code(app: &mut DefaultApp) -> Self {
        let contract =
            // ContractWrapper::new(execute, instantiate, query).with_reply(reply);
        ContractWrapper::new(execute, instantiate, query);

        let code_id = app.store_code(Box::new(contract));
        Self(code_id)
    }

    pub fn store_code(app: &mut App) -> Self {
        let contract = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(contract));
        Self(code_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn instantiate_with_voting_time(
        self,
        app: &mut App,
        sender: Addr,
        user1: Addr,
        user2: Addr,
        label: &str,
    ) -> AnyResult<MaciContract> {
        let round_info = RoundInfo {
            title: String::from("HackWasm Berlin"),
            description: String::from("Hack In Brelin"),
            link: String::from("https://baidu.com"),
        };
        let whitelist = Some(WhitelistBase {
            users: vec![
                WhitelistBaseConfig { addr: user1 },
                WhitelistBaseConfig { addr: user2 },
            ],
        });

        let start_time = Timestamp::from_nanos(1571797424879000000);
        let end_time = start_time.plus_minutes(11);
        let voting_time = VotingTime {
            start_time,
            end_time,
        };
        let circuit_type = Uint256::from_u128(0u128);
        let certification_system = Uint256::from_u128(0u128);
        MaciContract::instantiate(
            app,
            self,
            sender,
            round_info,
            whitelist,
            voting_time,
            circuit_type,
            certification_system,
            label,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn instantiate_with_wrong_voting_time(
        self,
        app: &mut App,
        sender: Addr,
        user1: Addr,
        user2: Addr,
        label: &str,
    ) -> AnyResult<MaciContract> {
        let round_info = RoundInfo {
            title: String::from("HackWasm Berlin"),
            description: String::from("Hack In Brelin"),
            link: String::from("https://baidu.com"),
        };
        let whitelist = Some(WhitelistBase {
            users: vec![
                WhitelistBaseConfig { addr: user1 },
                WhitelistBaseConfig { addr: user2 },
            ],
        });
        let voting_time = VotingTime {
            start_time: Timestamp::from_nanos(1571797429879300000),
            end_time: Timestamp::from_nanos(1571797424879000000),
        };
        let circuit_type = Uint256::from_u128(0u128);
        let certification_system = Uint256::from_u128(0u128);
        MaciContract::instantiate(
            app,
            self,
            sender,
            round_info,
            whitelist,
            voting_time,
            circuit_type,
            certification_system,
            label,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn instantiate_with_voting_time_and_no_whitelist(
        self,
        app: &mut App,
        sender: Addr,
        label: &str,
    ) -> AnyResult<MaciContract> {
        let round_info = RoundInfo {
            title: String::from("HackWasm Berlin"),
            description: String::from("Hack In Brelin"),
            link: String::from("https://baidu.com"),
        };
        let voting_time = VotingTime {
            start_time: Timestamp::from_nanos(1571797424879000000),
            end_time: Timestamp::from_nanos(1571797429879300000),
        };

        let circuit_type = Uint256::from_u128(0u128);
        let certification_system = Uint256::from_u128(0u128);
        MaciContract::instantiate(
            app,
            self,
            sender,
            round_info,
            None,
            voting_time,
            circuit_type,
            certification_system,
            label,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn instantiate_with_voting_time_isqv(
        self,
        app: &mut App,
        sender: Addr,
        user1: Addr,
        user2: Addr,
        label: &str,
    ) -> AnyResult<MaciContract> {
        let round_info = RoundInfo {
            title: String::from("HackWasm Berlin"),
            description: String::from("Hack In Brelin"),
            link: String::from("https://baidu.com"),
        };
        let whitelist = Some(WhitelistBase {
            users: vec![
                WhitelistBaseConfig { addr: user1 },
                WhitelistBaseConfig { addr: user2 },
            ],
        });
        let voting_time = VotingTime {
            start_time: Timestamp::from_nanos(1571797424879000000),
            end_time: Timestamp::from_nanos(1571797429879300000),
        };
        let circuit_type = Uint256::from_u128(1u128);
        let certification_system = Uint256::from_u128(0u128);
        MaciContract::instantiate(
            app,
            self,
            sender,
            round_info,
            whitelist,
            voting_time,
            circuit_type,
            certification_system,
            label,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn instantiate_with_voting_time_isqv_amaci(
        self,
        app: &mut App,
        sender: Addr,
        user1: Addr,
        user2: Addr,
        user3: Addr,
        label: &str,
    ) -> AnyResult<MaciContract> {
        let round_info = RoundInfo {
            title: String::from("HackWasm Berlin"),
            description: String::from("Hack In Brelin"),
            link: String::from("https://baidu.com"),
        };
        let whitelist = Some(WhitelistBase {
            users: vec![
                WhitelistBaseConfig { addr: user1 },
                WhitelistBaseConfig { addr: user2 },
                WhitelistBaseConfig { addr: user3 },
            ],
        });
        let start_time = Timestamp::from_nanos(1571797424879000000);
        let end_time = start_time.plus_minutes(11);
        let voting_time = VotingTime {
            start_time,
            end_time,
        };
        let circuit_type = Uint256::from_u128(1u128);
        let certification_system = Uint256::from_u128(0u128);
        MaciContract::instantiate_decative_and_add_new_key_zkey(
            app,
            self,
            sender,
            round_info,
            whitelist,
            voting_time,
            circuit_type,
            certification_system,
            label,
        )
    }
}

impl From<MaciCodeId> for u64 {
    fn from(code_id: MaciCodeId) -> Self {
        code_id.0
    }
}

#[derive(Debug, Clone)]
pub struct MaciContract(Addr);

// implement the contract real function, e.g. instantiate, functions in exec, query modules
impl MaciContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn new(addr: Addr) -> Self {
        MaciContract(addr)
    }

    #[allow(clippy::too_many_arguments)]
    #[track_caller]
    pub fn instantiate(
        app: &mut App,
        code_id: MaciCodeId,
        sender: Addr,
        round_info: RoundInfo,
        whitelist: Option<WhitelistBase>,
        voting_time: VotingTime,
        circuit_type: Uint256,
        certification_system: Uint256,
        label: &str,
    ) -> AnyResult<Self> {
        let parameters = MaciParameters {
            state_tree_depth: Uint256::from_u128(2u128),
            int_state_tree_depth: Uint256::from_u128(1u128),
            message_batch_size: Uint256::from_u128(5u128),
            vote_option_tree_depth: Uint256::from_u128(1u128),
        };
        let init_msg = InstantiateMsg {
            parameters,
            coordinator: PubKey {
                x: uint256_from_decimal_string(
                    "3557592161792765812904087712812111121909518311142005886657252371904276697771",
                ),
                y: uint256_from_decimal_string(
                    "4363822302427519764561660537570341277214758164895027920046745209970137856681",
                ),
            },
            max_vote_options: Uint256::from_u128(5u128),
            voice_credit_amount: Uint256::from_u128(100u128),
            pre_deactivate_root: Uint256::from_u128(0u128),
            round_info,
            whitelist,
            voting_time,
            circuit_type,
            certification_system,
            operator: operator(),
            admin: owner(),
        };

        app.instantiate_contract(
            code_id.0,
            Addr::unchecked(sender),
            &init_msg,
            &[],
            label,
            None,
        )
        .map(Self::from)
    }

    #[allow(clippy::too_many_arguments)]
    #[track_caller]
    pub fn instantiate_decative_and_add_new_key_zkey(
        app: &mut App,
        code_id: MaciCodeId,
        sender: Addr,
        round_info: RoundInfo,
        whitelist: Option<WhitelistBase>,
        voting_time: VotingTime,
        circuit_type: Uint256,
        certification_system: Uint256,
        label: &str,
    ) -> AnyResult<Self> {
        let parameters = MaciParameters {
            state_tree_depth: Uint256::from_u128(2u128),
            int_state_tree_depth: Uint256::from_u128(1u128),
            message_batch_size: Uint256::from_u128(5u128),
            vote_option_tree_depth: Uint256::from_u128(1u128),
        };
        let init_msg = InstantiateMsg {
            parameters,
            coordinator: PubKey {
                x: uint256_from_decimal_string(
                    "3557592161792765812904087712812111121909518311142005886657252371904276697771",
                ),
                y: uint256_from_decimal_string(
                    "4363822302427519764561660537570341277214758164895027920046745209970137856681",
                ),
            },
            max_vote_options: Uint256::from_u128(5u128),
            voice_credit_amount: Uint256::from_u128(100u128),
            pre_deactivate_root: Uint256::from_u128(0u128),
            round_info,
            whitelist,
            voting_time,
            circuit_type,
            certification_system,
            operator: operator(),
            admin: owner(),
        };

        app.instantiate_contract(
            code_id.0,
            Addr::unchecked(sender),
            &init_msg,
            &[],
            label,
            None,
        )
        .map(Self::from)
    }

    #[track_caller]
    pub fn sign_up(&self, app: &mut App, sender: Addr, pubkey: PubKey) -> AnyResult<AppResponse> {
        app.execute_contract(sender, self.addr(), &ExecuteMsg::SignUp { pubkey }, &[])
    }

    #[track_caller]
    pub fn publish_message(
        &self,
        app: &mut App,
        sender: Addr,
        message: MessageData,
        enc_pub_key: PubKey,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::PublishMessage {
                message,
                enc_pub_key,
            },
            &[],
        )
    }

    #[track_caller]
    pub fn set_round_info(&self, app: &mut App, sender: Addr) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::SetRoundInfo {
                round_info: RoundInfo {
                    title: String::from("TestRound2"),
                    description: String::from(""),
                    link: String::from("https://github.com"),
                },
            },
            &[],
        )
    }

    #[track_caller]
    pub fn set_empty_round_info(&self, app: &mut App, sender: Addr) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::SetRoundInfo {
                round_info: RoundInfo {
                    title: String::from(""),
                    description: String::from("Hello"),
                    link: String::from("https://github.com"),
                },
            },
            &[],
        )
    }

    #[track_caller]
    pub fn set_whitelist(&self, app: &mut App, sender: Addr) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::SetWhitelists {
                whitelists: WhitelistBase {
                    users: vec![
                        WhitelistBaseConfig { addr: user1() },
                        WhitelistBaseConfig { addr: user2() },
                    ],
                },
            },
            &[],
        )
    }

    #[track_caller]
    pub fn set_vote_option_map(&self, app: &mut App, sender: Addr) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::SetVoteOptionsMap {
                vote_option_map: vec![
                    String::from("did_not_vote"),
                    String::from("yes"),
                    String::from("no"),
                    String::from("no_with_veto"),
                    String::from("abstain"),
                ],
            },
            &[],
        )
    }

    #[track_caller]
    pub fn publish_deactivate_message(
        &self,
        app: &mut App,
        sender: Addr,
        message: MessageData,
        enc_pub_key: PubKey,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::PublishDeactivateMessage {
                message,
                enc_pub_key,
            },
            &[],
        )
    }

    #[track_caller]
    pub fn process_deactivate_message(
        &self,
        app: &mut App,
        sender: Addr,
        size: Uint256,
        new_deactivate_commitment: Uint256,
        new_deactivate_root: Uint256,
        proof: Groth16ProofType,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::ProcessDeactivateMessage {
                size,
                new_deactivate_commitment,
                new_deactivate_root,
                groth16_proof: proof,
            },
            &[],
        )
    }

    #[track_caller]
    pub fn add_key(
        &self,
        app: &mut App,
        sender: Addr,
        pubkey: PubKey,
        nullifier: Uint256,
        d: [Uint256; 4],
        proof: Groth16ProofType,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::AddNewKey {
                pubkey,
                nullifier,
                d,
                groth16_proof: proof,
            },
            &[],
        )
    }

    #[track_caller]
    pub fn pre_add_key(
        &self,
        app: &mut App,
        sender: Addr,
        pubkey: PubKey,
        nullifier: Uint256,
        d: [Uint256; 4],
        proof: Groth16ProofType,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::PreAddNewKey {
                pubkey,
                nullifier,
                d,
                groth16_proof: proof,
            },
            &[],
        )
    }

    #[track_caller]
    pub fn start_process(&self, app: &mut App, sender: Addr) -> AnyResult<AppResponse> {
        app.execute_contract(sender, self.addr(), &ExecuteMsg::StartProcessPeriod {}, &[])
    }

    #[track_caller]
    pub fn process_message(
        &self,
        app: &mut App,
        sender: Addr,
        new_state_commitment: Uint256,
        proof: Groth16ProofType,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::ProcessMessage {
                new_state_commitment,
                groth16_proof: proof,
            },
            &[],
        )
    }

    #[track_caller]
    pub fn stop_processing(&self, app: &mut App, sender: Addr) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::StopProcessingPeriod {},
            &[],
        )
    }

    #[track_caller]
    pub fn process_tally(
        &self,
        app: &mut App,
        sender: Addr,
        new_tally_commitment: Uint256,
        proof: Groth16ProofType,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::ProcessTally {
                new_tally_commitment,
                groth16_proof: proof,
            },
            &[],
        )
    }

    #[track_caller]
    pub fn stop_tallying(
        &self,
        app: &mut App,
        sender: Addr,
        results: Vec<Uint256>,
        salt: Uint256,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::StopTallyingPeriod { results, salt },
            &[],
        )
    }

    // #[track_caller]
    // pub fn grant(&self, app: &mut App, sender: Addr, sent: &[Coin]) -> AnyResult<AppResponse> {
    //     app.execute_contract(
    //         sender,
    //         self.addr(),
    //         &ExecuteMsg::Grant {
    //             max_amount: Uint128::from(10000000000000u128),
    //         },
    //         sent,
    //     )
    // }

    // #[track_caller]
    // pub fn revoke(&self, app: &mut App, sender: Addr) -> AnyResult<AppResponse> {
    //     app.execute_contract(sender, self.addr(), &ExecuteMsg::Revoke {}, &[])
    // }

    // #[track_caller]
    // pub fn bond(&self, app: &mut App, sender: Addr, sent: &[Coin]) -> AnyResult<AppResponse> {
    //     app.execute_contract(sender, self.addr(), &ExecuteMsg::Bond {}, sent)
    // }

    #[track_caller]
    pub fn withdraw(&self, app: &mut App, sender: Addr) -> AnyResult<AppResponse> {
        app.execute_contract(sender, self.addr(), &ExecuteMsg::Withdraw {}, &[])
    }

    pub fn msg_length(&self, app: &App) -> StdResult<Uint256> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::GetMsgChainLength {})
    }

    pub fn dmsg_length(&self, app: &App) -> StdResult<Uint256> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::GetDMsgChainLength {})
    }

    pub fn num_sign_up(&self, app: &App) -> StdResult<Uint256> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::GetNumSignUp {})
    }

    pub fn signuped(&self, app: &App, pubkey_x: Uint256) -> StdResult<Uint256> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::Signuped { pubkey_x })
    }

    pub fn vote_option_map(&self, app: &App) -> StdResult<Vec<String>> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::VoteOptionMap {})
    }

    pub fn max_vote_options(&self, app: &App) -> StdResult<Uint256> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::MaxVoteOptions {})
    }

    pub fn get_all_result(&self, app: &App) -> StdResult<Uint256> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::GetAllResult {})
    }

    pub fn get_voting_time(&self, app: &App) -> StdResult<VotingTime> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::GetVotingTime {})
    }

    pub fn get_period(&self, app: &App) -> StdResult<Period> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::GetPeriod {})
    }

    pub fn get_round_info(&self, app: &App) -> StdResult<RoundInfo> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::GetRoundInfo {})
    }

    pub fn query_total_feegrant(&self, app: &App) -> StdResult<Uint128> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::QueryTotalFeeGrant {})
    }

    pub fn query_delay_records(&self, app: &App) -> StdResult<DelayRecords> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::GetDelayRecords {})
    }

    pub fn query_admin(&self, app: &DefaultApp) -> StdResult<Addr> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::Admin {})
    }

    pub fn query_operator(&self, app: &DefaultApp) -> StdResult<Addr> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::Operator {})
    }

    pub fn query_round_info(&self, app: &DefaultApp) -> StdResult<RoundInfo> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::GetRoundInfo {})
    }
}

impl From<Addr> for MaciContract {
    fn from(value: Addr) -> Self {
        Self(value)
    }
}

pub fn user1() -> Addr {
    Addr::unchecked("0")
}

pub fn user2() -> Addr {
    Addr::unchecked("1")
}

pub fn user3() -> Addr {
    Addr::unchecked("2")
}

pub fn owner() -> Addr {
    Addr::unchecked("dora1qdagdkg9me4253h9qyvx83sd4gpta6rzh2fa0j")
}

pub fn operator() -> Addr {
    Addr::unchecked("operator")
}
