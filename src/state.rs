use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use cw_utils::Duration;

/// Smart contract configuration parameters
#[cw_serde]
pub struct Config {
    /// Owner of the contract
    pub owner: Addr,
    /// Address of the chain's staking module
    pub staking_module_address: Addr,
    /// Duration of each reward epoch
    pub epoch_duration: Duration
}

pub const CONFIG: Item<Config> = Item::new("configuration");