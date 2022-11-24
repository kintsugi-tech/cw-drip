use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Addr};
use cw_utils::Duration;

use crate::state::Config;

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of the chain's staking module
    pub staking_module_address: String,
    /// Duration of a single epoch for all the drip pool
    pub epoch_duration: Duration
}

//TODO: we can use directly Addr in CW20 for validation?
#[cw_serde]
pub enum TokenInfo {
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
    CreateDripPool {
        token_info: TokenInfo,
        min_staking_amount: Uint128,
        epochs_number: u64,
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Get the current config of the smart contract
    #[returns(ConfigResponse)]
    GetConfig {},
    // Get the vector of all participants
    #[returns(Vec<Addr>)]
    Participants {},
    // Get the vector of the drip tokens denom
    #[returns(Vec<String>)]
    DripTokens {},
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
pub struct ParticipantResponse {
    pub participants: Vec<Addr>
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


