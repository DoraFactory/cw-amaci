use crate::state::{
    MaciParameters, MessageData, PeriodStatus, PubKey, RoundInfo, VotingTime, Whitelist,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Uint256};

#[cw_serde]
pub struct InstantiateMsg {
    pub parameters: MaciParameters,
    pub coordinator: PubKey,
    pub admin: Addr,
    pub operator: Addr,
    // pub qtr_lib: QuinaryTreeRoot,
    // pub groth16_process_vkey: Groth16VKeyType,
    // pub groth16_tally_vkey: Groth16VKeyType,
    // pub groth16_deactivate_vkey: Groth16VKeyType,
    // pub groth16_add_key_vkey: Groth16VKeyType,
    pub max_vote_options: Uint256,
    pub voice_credit_amount: Uint256,

    pub round_info: RoundInfo,
    pub voting_time: VotingTime,
    pub whitelist: Option<Whitelist>,

    pub pre_deactivate_root: Uint256,

    // TODO: waiting add qv modal
    pub circuit_type: u64,         // <0: 1p1v | 1: pv>
    pub certification_system: u64, // <0: groth16 | 1: plonk>
}

#[cw_serde]
pub struct Groth16VKeyType {
    pub vk_alpha1: String,
    pub vk_beta_2: String,
    pub vk_gamma_2: String,
    pub vk_delta_2: String,
    pub vk_ic0: String,
    pub vk_ic1: String,
}

#[cw_serde]
pub struct Groth16ProofType {
    pub a: String,
    pub b: String,
    pub c: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // SetParams {
    //     state_tree_depth: Uint256,
    //     int_state_tree_depth: Uint256,
    //     message_batch_size: Uint256,
    //     vote_option_tree_depth: Uint256,
    // },
    SetRoundInfo {
        round_info: RoundInfo,
    },
    SetWhitelists {
        whitelists: Whitelist,
    },
    SetVoteOptionsMap {
        vote_option_map: Vec<String>,
    },
    // StartVotingPeriod {},
    SignUp {
        pubkey: PubKey, // user's pubkey
    },
    StartProcessPeriod {},
    // StopVotingPeriod {},
    PublishDeactivateMessage {
        message: MessageData,
        enc_pub_key: PubKey,
    },
    UploadDeactivateMessage {
        deactivate_message: Vec<Vec<Uint256>>,
    },
    ProcessDeactivateMessage {
        size: Uint256,
        new_deactivate_commitment: Uint256,
        new_deactivate_root: Uint256,
        groth16_proof: Groth16ProofType,
    },
    AddNewKey {
        pubkey: PubKey,
        nullifier: Uint256,
        d: [Uint256; 4],
        groth16_proof: Groth16ProofType,
    },
    PreAddNewKey {
        pubkey: PubKey,
        nullifier: Uint256,
        d: [Uint256; 4],
        groth16_proof: Groth16ProofType,
    },
    PublishMessage {
        message: MessageData,
        enc_pub_key: PubKey,
    },
    ProcessMessage {
        new_state_commitment: Uint256,
        groth16_proof: Groth16ProofType,
    },
    StopProcessingPeriod {},
    ProcessTally {
        new_tally_commitment: Uint256,
        groth16_proof: Groth16ProofType,
    },
    StopTallyingPeriod {
        results: Vec<Uint256>,
        salt: Uint256,
    },
    Grant {
        max_amount: Uint128,
    },
    Revoke {},
    Bond {},
    Withdraw {
        amount: Option<Uint128>,
    },
}

#[cw_serde]
pub struct Period {
    pub status: PeriodStatus,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Addr)]
    Admin {},

    #[returns(Addr)]
    Operator {},

    #[returns(RoundInfo)]
    GetRoundInfo {},

    #[returns(VotingTime)]
    GetVotingTime {},

    #[returns(Period)]
    GetPeriod {},

    #[returns(Uint256)]
    GetNumSignUp {},

    #[returns(Uint256)]
    GetMsgChainLength {},

    #[returns(Uint256)]
    GetDMsgChainLength {},

    #[returns(Uint256)]
    GetProcessedDMsgCount {},

    #[returns(Uint256)]
    GetProcessedMsgCount {},

    #[returns(Uint256)]
    GetProcessedUserCount {},

    #[returns(Uint256)]
    GetResult { index: Uint256 },

    #[returns(Uint256)]
    GetAllResult {},

    #[returns(Uint256)]
    GetStateIdxInc { address: Addr },

    #[returns(Uint256)]
    GetVoiceCreditBalance { index: Uint256 },

    #[returns(Uint256)]
    GetVoiceCreditAmount {},

    #[returns(Whitelist)]
    WhiteList {},
    /// Checks permissions of the caller on this proxy.
    /// If CanExecute returns true then a call to `Execute` with the same message,
    /// before any further state changes, should also succeed.
    #[returns(bool)]
    IsWhiteList { sender: Addr },

    // #[returns(Uint256)]
    // WhiteBalanceOf { sender: String },
    #[returns(Vec<String>)]
    VoteOptionMap {},

    #[returns(Uint256)]
    MaxVoteOptions {},

    #[returns(Uint128)]
    QueryTotalFeeGrant {},

    #[returns(u64)]
    QueryCircuitType {},

    #[returns(u64)]
    QueryCertSystem {},

    #[returns(Uint256)]
    QueryPreDeactivateRoot {},
}

#[cw_serde]
pub struct InstantiationData {
    pub caller: Addr,
    pub parameters: MaciParameters,
    pub coordinator: PubKey,
    pub admin: Addr,
    pub operator: Addr,
    pub max_vote_options: Uint256,
    pub voice_credit_amount: Uint256,
    pub round_info: RoundInfo,
    pub voting_time: VotingTime,
    pub pre_deactivate_root: Uint256,
    pub circuit_type: String,
    pub certification_system: String,
}
