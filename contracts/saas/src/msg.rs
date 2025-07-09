use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Uint256};
use cw_amaci::msg::WhitelistBase;
use cw_amaci::state::{RoundInfo, VotingTime};

use crate::state::{Config, ConsumptionRecord, FeeGrantRecord, OperatorInfo};

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
}

#[cw_serde]
pub struct MigrateMsg {}
