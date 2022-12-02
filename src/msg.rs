use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Addr};
use cw_utils::Duration;

use crate::state::{Config, DripPool};

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of the chain's staking module
    pub staking_module_address: String,
    /// Minimum native token staked to participate
    pub min_staking_amount: Uint128,
    /// Duration of a single epoch for all the drip pool
    pub epoch_duration: Duration,
}

/// Unvalidated drip token
#[cw_serde]
pub enum DripToken {
    Native {
        /// Token denom
        denom: String,
        /// Initial amount of the drip pool
        initial_amount: Uint128,
    },
    CW20 {
        /// CW20 smart contract address
        address: String,
        /// Initial amount of the drip pool
        initial_amount: Uint128,
    }
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Participate to the drip distribution
    Participate {},
    /// Remove participation to the drip distribution. No more shares
    /// will be accrued.
    RemoveParticipation {},
    /// Create a distribution drip pool
    CreateDripPool {
        token_info: DripToken,
        tokens_per_epoch: Uint128,
        epochs_number: u64,
    },
    UpdateDripPool {},
    RemoveDripPool {},
    /// Compute and distribute active drip pools shares to
    /// participants
    DistributeShares {},
    WithdrawToken {},
    WithdrawTokens {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get the current smart contract config
    #[returns(ConfigResponse)]
    Config {},
    /// Get the vector of participants
    #[returns(ParticipantsResponse)]
    Participants {},
    /// Get the vector of drip tokens denom
    #[returns(DripTokensResponse)]
    DripTokens {},
    /// Get info of a specific drip pool
    #[returns(DripPoolResponse)]
    DripPool { token: String },
    /// Get all drip pools
    #[returns(DripPoolsResponse)]
    DripPools {},
}

// Query response structures
#[cw_serde]
pub struct ConfigResponse {
    pub config: Config
}

#[cw_serde]
pub struct ParticipantsResponse {
    pub participants: Vec<Addr>
}

#[cw_serde]
pub struct DripTokensResponse {
    pub drip_tokens: Vec<String>
}

#[cw_serde]
pub struct DripPoolResponse {
    pub drip_pool: Option<DripPool>
}

#[cw_serde]
pub struct DripPoolsResponse {
    pub drip_pools: Vec<DripPool>
}

#[cw_serde]
pub struct VotingPowerAtHeightReponse {
    pub power: Uint128,
    pub height: u64
}

#[cw_serde]
pub struct TotalPowerAtHeightResponse {
    pub power: Uint128,
    pub height: u64
}

#[cw_serde]
pub struct AddParticipantResponse {
    pub address: Addr,
    pub eligible: bool,
}


