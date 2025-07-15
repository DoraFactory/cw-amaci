use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub registry_contract: Option<Addr>,
    pub denom: String,
}

impl Config {
    pub fn is_admin(&self, addr: &Addr) -> bool {
        self.admin == *addr
    }
}

#[cw_serde]
pub struct OperatorInfo {
    pub address: Addr,
    pub added_at: Timestamp,
    pub active: bool,
}

#[cw_serde]
pub struct FeeGrantRecord {
    pub grantee: Addr,
    pub amount: Uint128,
    pub granted_at: Timestamp,
    pub granted_by: Addr,
}

// 新增: MACI 合约信息跟踪
#[cw_serde]
pub struct MaciContractInfo {
    pub contract_address: Addr,
    pub creator_operator: Addr,
    pub round_title: String,
    pub created_at: Timestamp,
    pub code_id: u64,
    pub creation_fee: Uint128,
}

// Storage items
pub const CONFIG: Item<Config> = Item::new("config");
pub const OPERATORS: Map<&Addr, OperatorInfo> = Map::new("operators");
pub const TOTAL_BALANCE: Item<Uint128> = Item::new("total_balance");
pub const FEEGRANT_RECORDS: Map<&Addr, FeeGrantRecord> = Map::new("feegrant_records");

// 新增: MACI 合约跟踪存储
pub const MACI_CONTRACT_COUNTER: Item<u64> = Item::new("maci_contract_counter");
pub const MACI_CONTRACTS: Map<u64, MaciContractInfo> = Map::new("maci_contracts");
pub const MACI_CONTRACTS_BY_OPERATOR: Map<(&Addr, u64), bool> =
    Map::new("maci_contracts_by_operator");
