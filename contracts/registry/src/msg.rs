use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Uint256};

use crate::state::PubKey;

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
pub enum ExecuteMsg {
    Register {
        pubkey: PubKey,
    },
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
    // // TODO: only operator can slash the token, 不能一直slash token，所以需要设置一个 slash 之后的 状态
    // // 并且，我们可能需要设置一个定期处理的机制，比如一个月一个epoch
    // // 另外，关于投票周期，不能太长也不能太短。
    // Slash {
    //     operator: Addr,
    //     amount: Uint128,
    //     proof: String,
    // },
}

#[cw_serde]
pub enum ReceiveMsg {
    /// Only valid cw20 message is to bond the tokens
    Bond {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
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

    #[returns(PubKey)]
    GetMaciOperatorPubkey { address: Addr },
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
