use cosmwasm_std::{OverflowError, StdError, Uint128};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Operator not found")]
    OperatorNotFound {},

    #[error("Operator already exists")]
    OperatorAlreadyExists {},

    #[error("Insufficient balance in SaaS contract")]
    InsufficientBalance {},

    #[error("Invalid address prefix, expected: {expected}, got: {actual}")]
    InvalidAddressPrefix { expected: String, actual: String },

    #[error("Invalid address: {address}")]
    InvalidAddress { address: String },

    #[error("Insufficient funds for creating round. Required: {required}, available: {available}")]
    InsufficientFundsForRound {
        required: Uint128,
        available: Uint128,
    },

    #[error("No registry contract set")]
    NoRegistryContract {},

    #[error("Invalid Oracle MACI parameters: {reason}")]
    InvalidOracleMaciParameters { reason: String },

    #[error("Message serialization failed: {msg}")]
    SerializationError { msg: String },

    #[error("No funds sent")]
    NoFunds {},

    #[error("Feegrant amount must be greater than zero")]
    InvalidFeegrantAmount {},

    #[error("Cannot withdraw zero amount")]
    InvalidWithdrawAmount {},

    #[error("Address list cannot be empty")]
    EmptyAddressList {},

    #[error("Value too large for conversion")]
    ValueTooLarge {},

    #[error("Payment error: {0}")]
    Payment(#[from] PaymentError),

    #[error("Overflow error: {0}")]
    Overflow(#[from] OverflowError),

    #[error("Contract instantiation failed")]
    ContractInstantiationFailed {},
}
