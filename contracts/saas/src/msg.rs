use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Coin, Uint128, Uint256};
use cw_amaci::msg::WhitelistBase;
use cw_amaci::state::{RoundInfo, VotingTime};

use crate::state::{Config, ConsumptionRecord, FeeGrantRecord, OperatorInfo};

// 从 maci 合约导入需要的类型
#[cw_serde]
pub struct MaciParameters {
    pub state_tree_depth: Uint256,
    pub int_state_tree_depth: Uint256,
    pub message_batch_size: Uint256,
    pub vote_option_tree_depth: Uint256,
}

#[cw_serde]
pub struct PubKey {
    pub x: Uint256,
    pub y: Uint256,
}

#[cw_serde]
pub struct QuinaryTreeRoot {
    pub zeros: [Uint256; 9],
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
pub struct PlonkVKeyType {
    pub n: usize,
    pub num_inputs: usize,
    pub selector_commitments: Vec<String>,
    pub next_step_selector_commitments: Vec<String>,
    pub permutation_commitments: Vec<String>,
    pub non_residues: Vec<String>,
    pub g2_elements: Vec<String>,
}

#[cw_serde]
pub struct WhitelistConfig {
    pub addr: String,
    pub balance: Uint256,
}

#[cw_serde]
pub struct Whitelist {
    pub users: Vec<WhitelistConfig>,
}

#[cw_serde]
pub struct MaciVotingTime {
    pub start_time: Option<cosmwasm_std::Timestamp>,
    pub end_time: Option<cosmwasm_std::Timestamp>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub registry_contract: Option<Addr>,
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Admin management
    UpdateConfig {
        admin: Option<Addr>,
        registry_contract: Option<Addr>,
        denom: Option<String>,
    },

    // Operator management
    AddOperator {
        operator: Addr,
    },
    RemoveOperator {
        operator: Addr,
    },

    // Deposit/Withdraw functions
    Deposit {},
    Withdraw {
        amount: Uint128,
        recipient: Option<Addr>,
    },

    // Feegrant functions
    BatchFeegrant {
        recipients: Vec<Addr>,
        amount: Uint128,
    },
    BatchFeeGrantToOperators {
        amount: Uint128,
    },

    // Create AMACI round
    CreateAmaciRound {
        max_voter: Uint256,
        max_option: Uint256,
        voice_credit_amount: Uint256,
        round_info: RoundInfo,
        voting_time: VotingTime,
        whitelist: Option<WhitelistBase>,
        pre_deactivate_root: Uint256,
        circuit_type: Uint256,
        certification_system: Uint256,
    },

    // Create MACI round (direct contract instantiation)
    CreateMaciRound {
        maci_code_id: u64,
        parameters: MaciParameters,
        coordinator: PubKey,
        qtr_lib: QuinaryTreeRoot,
        groth16_process_vkey: Option<Groth16VKeyType>,
        groth16_tally_vkey: Option<Groth16VKeyType>,
        plonk_process_vkey: Option<PlonkVKeyType>,
        plonk_tally_vkey: Option<PlonkVKeyType>,
        max_vote_options: Uint256,
        round_info: RoundInfo,
        voting_time: Option<MaciVotingTime>,
        whitelist: Option<Whitelist>,
        circuit_type: Uint256,
        certification_system: Uint256,
        admin_override: Option<Addr>, // 可选的管理员地址覆盖
        label: String,                // 合约标签
    },

    // Execute other contracts
    ExecuteContract {
        contract_addr: String,
        msg: Binary,
        funds: Vec<Coin>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},

    #[returns(Vec<OperatorInfo>)]
    Operators {},

    #[returns(bool)]
    IsOperator { address: Addr },

    #[returns(Uint128)]
    Balance {},

    #[returns(Vec<ConsumptionRecord>)]
    ConsumptionRecords {
        start_after: Option<u64>,
        limit: Option<u32>,
    },

    #[returns(Vec<FeeGrantRecord>)]
    FeeGrantRecords {
        start_after: Option<Addr>,
        limit: Option<u32>,
    },

    #[returns(Vec<ConsumptionRecord>)]
    OperatorConsumptionRecords {
        operator: Addr,
        start_after: Option<u64>,
        limit: Option<u32>,
    },

    #[returns(Vec<crate::state::MaciContractInfo>)]
    MaciContracts {
        start_after: Option<u64>,
        limit: Option<u32>,
    },

    #[returns(Vec<crate::state::MaciContractInfo>)]
    OperatorMaciContracts {
        operator: Addr,
        start_after: Option<u64>,
        limit: Option<u32>,
    },

    #[returns(Option<crate::state::MaciContractInfo>)]
    MaciContract { contract_id: u64 },
}

#[cw_serde]
pub struct MigrateMsg {}
