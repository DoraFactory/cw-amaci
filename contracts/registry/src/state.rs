use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint256};
use cw4::TOTAL_KEY;
use cw_amaci::state::PubKey;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    /// denom of the token to stake
    pub denom: String,
    pub min_deposit_amount: Uint128,
    pub slash_amount: Uint128,
}

#[cw_serde]
pub struct Admin {
    pub admin: Addr,
}

impl Admin {
    pub fn is_admin(&self, addr: impl AsRef<str>) -> bool {
        let addr = addr.as_ref();
        self.admin.as_ref() == addr
    }
}

pub const ADMIN: Item<Admin> = Item::new("admin");
pub const OPERATOR: Item<Addr> = Item::new("operator");
pub const CONFIG: Item<Config> = Item::new("config");
pub const AMACI_CODE_ID: Item<u64> = Item::new("amaci_code_id");
pub const TOTAL: Item<u128> = Item::new(TOTAL_KEY);
pub const MACI_OPERATOR_SET: Map<&Addr, Uint128> = Map::new("maci_operator_set");
pub const MACI_DEACTIVATE_OPERATOR: Map<&Addr, Addr> = Map::new("maci_deactivate_operator"); // contract_address - operator_address

pub const MACI_OPERATOR_PUBKEY: Map<&Addr, PubKey> = Map::new("maci_operator_pubkey"); // operator_address - coordinator_pubkey
pub const COORDINATOR_PUBKEY_MAP: Map<&(Vec<u8>, Vec<u8>), u64> =
    Map::new("coordinator_pubkey_map"); //
