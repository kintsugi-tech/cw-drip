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
    pub epoch_duration: Duration
}

#[cw_serde]
pub enum DripToken {
    Native {
        denom: String,
        initial_amount: Uint128,
    },
    CW20 {
        address: String,
        initial_amount: Uint128,
    }
}

#[cw_serde]
pub enum ExecuteMsg {
    Participate {},
    RemoveParticipation {},
    CreateDripPool {
        token_info: DripToken,
        epochs_number: u64,
    },
    UpdateDripPool {},
    RemoveDripPool {},
    DistributeShares {},
    WithdrawToken {},
    WithdrawTokens {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get the current config of the smart contract
    #[returns(ConfigResponse)]
    Config {},
    /// Get the vector of all participants
    #[returns(ParticipantsResponse)]
    Participants {},
    /// Get the vector of the drip tokens denom
    #[returns(DripTokensResponse)]
    DripTokens {},
    /// Get info of a drip pool
    #[returns(DripPoolResponse)]
    DripPool {
        token: String
    },
    /// Get all drip pools
    #[returns(DripPoolsResponse)]
    DripPools {},
    /// Get the total number of drip pools
    #[returns(VotingPowerAtHeightReponse)]
    VotingPowerAtHeight {
        address: String,
        height: Option<u64>,
    },
    #[returns(TotalPowerAtHeightResponse)]
    TotalPowerAtHeight {
        height: Option<u64>,
    },
}

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


