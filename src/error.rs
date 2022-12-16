use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq))] // Only neeed while testing.
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.

    #[error("sender is already a participant")]
    AlreadyParticipant {},

    #[error("drip pool already exists")]
    DripPoolAlreadyExists {},

    #[error("smart contract has not enough tokens, missing: [{token}] [{amount}]")]
    NoFundedContract { token: String, amount: Uint128 },

    #[error("zero token pool is not allowed")]
    ZeroTokenPool {},

    #[error("rip pool for token [{token}] not found.")]
    DripPoolNotFound { token: String },

    #[error("no active drip pool")]
    ZeroActiveDripPool {},

    #[error("initial tokens amount [{tokens_amount}] does not coincide with epochs_number X tokens_per_epoch: [{total_tokens}]")]
    WrongTokensAmount { tokens_amount: Uint128, total_tokens: Uint128},

    #[error("wait for distribution time")]
    NoDistributionTime {},

    #[error("no shares for this address")]
    NoShares {},

    #[error("the minimum number of epochs is 1")]
    LessThanOneEpoch {},

    #[error("drip pool should be unactive")]
    InvalidActiveDripPool,

    #[error("drip pool has not enough funds to distribute")]
    DripPoolHasNotENoughFunds,

    #[error("no tokens to withdraw")]
    NoTokensToWithdraw {},
}
