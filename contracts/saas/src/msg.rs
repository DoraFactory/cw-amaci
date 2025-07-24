use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Uint256};
use cw_amaci::state::RoundInfo;

use crate::state::{Config, OperatorInfo};

#[cw_serde]
pub struct PubKey {
    pub x: Uint256,
    pub y: Uint256,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub registry_contract: Option<Addr>,
    pub denom: String,
    pub oracle_maci_code_id: u64,
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

    UpdateOracleMaciCodeId {
        code_id: u64,
    },

    // Create Oracle MACI round
    CreateOracleMaciRound {
        coordinator: PubKey,
        max_voters: u128,
        vote_option_map: Vec<String>,
        round_info: RoundInfo,
        start_time: cosmwasm_std::Timestamp,
        end_time: cosmwasm_std::Timestamp,
        circuit_type: Uint256,
        certification_system: Uint256,
        whitelist_backend_pubkey: String,
        // 以下参数在合约内部写死:
        // whitelist_ecosystem: "doravota"
        // whitelist_snapshot_height: 0
        // whitelist_voting_power_args: slope 模式 (1人1票)
    },

    // Oracle MACI management
    SetRoundInfo {
        contract_addr: String,
        round_info: RoundInfo,
    },
    SetVoteOptionsMap {
        contract_addr: String,
        vote_option_map: Vec<String>,
    },

    // Oracle MACI feegrant management
    GrantToVoter {
        contract_addr: String,
        grantee: Addr,
        base_amount: Uint128,
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

    #[returns(u64)]
    OracleMaciCodeId {},
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct InstantiationData {
    pub addr: Addr,
}
