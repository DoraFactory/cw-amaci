use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Uint256};

#[cw_serde]
pub struct InstantiateMsg {
    /// denom of the token to stake
    pub denom: String,

    pub min_deposit_amount: Uint128,

    pub slash_amount: Uint128,

    // admin can only bond/withdraw token
    pub admin: Addr,

    // operator can add whitelist address
    pub operator: Addr,
}

#[cw_serde]
pub enum RegisterExecuteMsg {
    Register {},
    Deregister {},
    UploadDeactivateMessage {
        contract_address: Addr,
        deactivate_message: Vec<Vec<Uint256>>,
    },
    ChangeParams {
        min_deposit_amount: Uint128,
        slash_amount: Uint128,
    },
    ChangeOperator {
        address: Addr,
    },
}

#[cw_serde]
pub enum ReceiveMsg {
    /// Only valid cw20 message is to bond the tokens
    Bond {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum RegisterQueryMsg {
    #[returns(ConfigResponse)]
    GetConfig {},

    #[returns(AdminResponse)]
    Admin {},

    #[returns(Addr)]
    Operator {},

    #[returns(u128)]
    GetTotal {},

    #[returns(bool)]
    IsMaciOperator { address: Addr },

    #[returns(Vec<Vec<String>>)]
    GetMaciDeactivate { contract_address: Addr },

    #[returns(Addr)]
    GetMaciOperator { contract_address: Addr },
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
