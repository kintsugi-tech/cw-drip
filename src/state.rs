use std::collections::HashMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::{Item, Map};
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

#[cw_serde]
pub struct DripPool { 
    address: Option<Addr>,
    initial_amount: Uint128,
    actual_amount: Uint128,
    issued_shares: Uint128,
    min_staking_amount: Uint128,
    epochs_number: u64,
    current_epoch: u64,
}

pub const CONFIG: Item<Config> = Item::new("configuration");

// Structure to save all the addresses that opted-in and if they are
// eligible for the current epoch.
pub const PARTICIPANTS_SHARES: Map<(&Addr, &String), Uint128> = Map::new("participants_shares");
pub const DRIP_POOLS: Map<&String, DripPool> = Map::new("drip_pools");