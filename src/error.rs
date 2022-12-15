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

    #[error("AlreadyPartecipant")]
    AlreadyParticipant {},

    #[error("DripAlreadyExists")]
    DripPoolAlreadyExists {},

    #[error("Smart contract has not enough tokens. Missing: [{token}] [{amount}]")]
    NoFundedContract { token: String, amount: Uint128 },

    #[error("ZeroTokenPool")]
    ZeroTokenPool {},

    #[error("Drip pool for token [{token}] not found.")]
    DripPoolNotFound { token: String },

    #[error("No drip pool active.")]
    ZeroActiveDripPool {},

    #[error("Tokens initial amount [{tokens_amount}] does not coincide with epochs number times tokens per epoch [{total_tokens}]")]
    WrongTokensAmount { tokens_amount: Uint128, total_tokens: Uint128},

    #[error("Wait for distribution time")]
    NoDistributionTime {},

    #[error("No shares for this address")]
    NoShares {},

    #[error("The sender has not shares to withdraw tokens")]
    AddressNotAssociatedToShares,

    #[error("The minimum number of epochs is 1.")]
    LessThanOneEpoch {  },
}
