use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Uint256};

use cw_amaci::state::{PubKey, RoundInfo, VotingTime, Whitelist};

use crate::state::ValidatorSet;

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
        whitelist: Option<Whitelist>,
        pre_deactivate_root: Uint256,
        circuit_type: u64,
        certification_system: u64,
    },
    // ChangeParams {
    //     min_deposit_amount: Uint128,
    //     slash_amount: Uint128,
    // },
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
    // #[returns(u64)]
    // GetNewState {},
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
