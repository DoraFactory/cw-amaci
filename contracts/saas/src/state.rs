use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub registry_contract: Option<Addr>,
    pub denom: String,
}

#[cw_serde]
pub struct OperatorInfo {
    pub address: Addr,
    pub added_at: Timestamp,
    pub active: bool,
}

#[cw_serde]
pub struct ConsumptionRecord {
    pub operator: Addr,
    pub action: String,
    pub amount: Uint128,
    pub timestamp: Timestamp,
    pub description: String,
}

#[cw_serde]
pub struct FeeGrantRecord {
    pub grantee: Addr,
    pub amount: Uint128,
    pub granted_at: Timestamp,
    pub granted_by: Addr,
}

// Storage keys
pub const CONFIG: Item<Config> = Item::new("config");
pub const OPERATORS: Map<&Addr, OperatorInfo> = Map::new("operators");
pub const CONSUMPTION_RECORDS: Map<u64, ConsumptionRecord> = Map::new("consumption_records");
pub const CONSUMPTION_COUNTER: Item<u64> = Item::new("consumption_counter");
pub const FEEGRANT_RECORDS: Map<&Addr, FeeGrantRecord> = Map::new("feegrant_records");
pub const TOTAL_BALANCE: Item<Uint128> = Item::new("total_balance");

impl Config {
    pub fn is_admin(&self, addr: &Addr) -> bool {
        self.admin == *addr
    }
}
