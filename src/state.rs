use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use cw_utils::{Duration, Expiration};

/// Smart contract configuration parameters
#[cw_serde]
pub struct Config {
    /// Owner of the contract
    pub owner: Addr,
    /// Address of the chain's staking module
    pub staking_module_address: Addr,
    // Minimum amount of native token staked to be allowed to participate 
    pub min_staking_amount: Uint128,
    /// Duration of each reward epoch
    pub epoch_duration: Duration,
    /// Creation block
    pub last_distribution_time: Option<Duration>,
    /// Epoch number of the last distribution
    pub next_distribution_time: Expiration, 
}

/// Drip pool information saved on storage
#[cw_serde]
pub struct DripPool {
    /// Token to be distributed
    pub drip_token : CheckedDripToken,
    /// Remaining amount to distribute
    pub actual_amount: Uint128,
    /// Tokens that can be withdrawed
    pub tokens_to_withdraw: Uint128,
    /// Tokens to distribute at every epoch
    pub tokens_per_epoch: Uint128,
    /// Shares issued to participants
    pub issued_shares: Uint128,
    /// Total distribution epoch
    pub epochs_number: u64,
    /// Current distribution epoch. After distributing the first time
    /// current_epoch will be 1 and so on.
    pub current_epoch: u64,
}

/// Drip token variants after initial amount check
#[cw_serde]
pub enum CheckedDripToken {
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

/// Structure for managing participant shares for one drip token
#[cw_serde]
pub struct DripPoolShares {
    /// Denom or address of the token
    pub token: String,
    /// Total amount of shares
    pub total_shares: Uint128
}

// Configuration.
pub const CONFIG: Item<Config> = Item::new("config");

// All participants to the drip
pub const PARTICIPANTS: Item<Vec<Addr>> = Item::new("participants");

// All drip tokens of active drip pools
pub const DRIP_TOKENS: Item<Vec<String>> = Item::new("drip_tokens");

// Participants shares of every drip pool
pub const PARTICIPANTS_SHARES: Map<&Addr, Vec<DripPoolShares>> = Map::new("participants_shares");

// Drip pools info
pub const DRIP_POOLS: Map<String, DripPool> = Map::new("drip_pools");



impl CheckedDripToken {
    // Getter for initial amount of the drip token
    pub fn get_initial_amount(&self) -> Uint128 {
        match self {
            CheckedDripToken::Native {denom: _, initial_amount} => {
                return initial_amount.clone()
            }
            CheckedDripToken::CW20 { address: _, initial_amount } => {
                return initial_amount.clone()
            }
        }
    }
    
    // Getter for the drip token unique identifier
    pub fn get_token(&self) -> String {
        match self {
            CheckedDripToken::Native {denom, initial_amount: _} => {
                return denom.clone()
            }
            // We have to use address because is the only unique
            CheckedDripToken::CW20 { address, initial_amount: _ } => {
                return address.clone()
            }
        }
    }


}

#[cfg(test)]
#[test]
fn test_get_initial_amount() {
    let token = CheckedDripToken::Native { denom: "ujuno".to_string(), initial_amount: Uint128::zero() };
    let amount = token.get_initial_amount();
    assert_eq!(amount, Uint128::zero());

    let token = CheckedDripToken::Native { denom: "ujuno".to_string(), initial_amount: Uint128::new(1_000) };
    let amount = token.get_initial_amount();
    assert_ne!(amount, Uint128::zero());
    assert_eq!(amount, Uint128::new(1000));
}
