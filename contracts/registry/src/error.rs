use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

use cw_controllers::{AdminError, HookError};

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("expected {expected} but got {actual}")]
    InvalidAmount { expected: u128, actual: u128 },

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    Hook(#[from] HookError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("This Maci Operator is Already Register")]
    ExistedMaciOperator {},

    #[error("Insufficient deposit amount, minimum deposit {min_deposit_amount}")]
    InsufficientDeposit { min_deposit_amount: Uint128 },

    #[error("No claims that can be released currently")]
    NothingToClaim {},

    #[error("Already claimed")]
    AlreadyClaimed {},

    #[error("Admin bond token is not enough")]
    BondTokenNotEnough {},

    #[error("Must send '{0}' to stake")]
    MissingDenom(String),

    #[error("Sent unsupported denoms, must send '{0}' to stake")]
    ExtraDenoms(String),

    #[error("Must send valid address to stake")]
    InvalidDenom(String),

    #[error("Missed address or denom")]
    MixedNativeAndCw20(String),

    #[error("No funds sent")]
    NoFunds {},

    #[error("No data in ReceiveMsg")]
    NoData {},

    #[error("No matched-size circuits")]
    NoMatchedSizeCircuit,

    #[error("Un recognized reply id {id}")]
    UnRecognizedReplyIdErr { id: u64 },

    #[error("Data missing")]
    DataMissingErr {},

    #[error("Invalid pubkey length. Must be 33 bytes.")]
    InvalidPubkeyLength {},

    #[error("This pubkey is already existed.")]
    PubkeyExisted {},

    #[error("Not set operator pubkey.")]
    NotSetOperatorPubkey,
}
