use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Uint256};

use cw_amaci::{
    msg::WhitelistBase,
    state::{PubKey, RoundInfo, VotingTime},
};

use crate::state::{CircuitChargeConfig, ValidatorSet};

#[cw_serde]
pub struct InstantiateMsg {
    // /// denom of the token to stake
    // pub denom: String,

    // pub min_deposit_amount: Uint128,

    // pub slash_amount: Uint128,

    // admin can only bond/withdraw token
    pub admin: Addr,

    // operator can add whitelist address
    pub operator: Addr,

    pub amaci_code_id: u64,
}

// Sponsor module message types
#[derive(Clone, PartialEq, prost::Message)]
pub struct ProtoCoin {
    #[prost(string, tag = "1")]
    pub denom: String,
    #[prost(string, tag = "2")]
    pub amount: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct MsgSetSponsor {
    #[prost(string, tag = "1")]
    pub creator: String,
    #[prost(string, tag = "2")]
    pub contract_address: String,
    #[prost(bool, tag = "3")]
    pub is_sponsored: bool,
    #[prost(message, repeated, tag = "4")]
    pub max_grant_per_user: Vec<ProtoCoin>,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetMaciOperator {
        operator: Addr,
    },
    SetMaciOperatorPubkey {
        pubkey: PubKey,
    },
    SetMaciOperatorIdentity {
        identity: String,
    },
    CreateRound {
        operator: Addr,
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
    SetValidators {
        addresses: ValidatorSet,
    },
    RemoveValidator {
        address: Addr,
    },
    UpdateAmaciCodeId {
        amaci_code_id: u64,
    },
    ChangeOperator {
        address: Addr,
    },
    ChangeChargeConfig {
        config: CircuitChargeConfig,
    },
    RegisterSponsor {
        contract_address: String,
        is_sponsored: bool,
        max_grant_amount: Uint128,
        denom: String,
    },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AdminResponse)]
    Admin {},

    #[returns(Addr)]
    Operator {},

    #[returns(bool)]
    IsMaciOperator { address: Addr },

    #[returns(bool)]
    IsValidator { address: Addr },

    #[returns(ValidatorSet)]
    GetValidators {},

    #[returns(Addr)]
    GetValidatorOperator { address: Addr },

    #[returns(PubKey)]
    GetMaciOperatorPubkey { address: Addr },

    #[returns(String)]
    GetMaciOperatorIdentity { address: Addr },

    #[returns(CircuitChargeConfig)]
    GetCircuitChargeConfig {},
}

#[cw_serde]
pub struct AdminResponse {
    pub admin: Addr,
}

#[cw_serde]
pub struct ConfigResponse {
    pub denom: String,
    pub min_deposit_amount: Uint128,
    pub slash_amount: Uint128,
}

#[cw_serde]
pub struct InstantiationData {
    pub addr: Addr,
}
