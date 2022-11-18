use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw_utils::Duration;

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of the chain's staking module
    pub staking_module_address: String,
    pub epoch_duration: Duration
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VotingPowerAtHeightReponse)]
    VotingPowerAtHeight {
        address: String,
        height: Option<u64>,
    },
    #[returns(TotalPowerAtHeightResponse)]
    TotalPowerAtHeight {
        height: Option<u64>,
    }
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




