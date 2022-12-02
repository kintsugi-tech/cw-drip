use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Deps, Env, BlockInfo};
use cw20::{Cw20QueryMsg};
use cw_storage_plus::{Item, Map};
use cw_utils::{Duration, Expiration};

use crate::{ContractError, msg::DripToken};

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
    /// Tokens to distribute at every epoch
    pub tokens_per_epoch: Uint128,
    /// Shares issued to participants
    pub issued_shares: Uint128,
    /// Total distribution epoch
    pub epochs_number: u64,
    /// Current distribution epoch
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
        /// Token symbol
        symbol: String,
        /// CW20 smart contract address
        address: Addr,
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
    pub totale_shares: Uint128
}

pub const CONFIG: Item<Config> = Item::new("config");

// Storage for the list of all participants.
pub const PARTICIPANTS: Item<Vec<Addr>> = Item::new("participants");

// Storage for the list of all drip tokens.
pub const DRIP_TOKENS: Item<Vec<String>> = Item::new("drip_tokens");

pub const PARTICIPANTS_SHARES: Map<&Addr, Vec<DripPoolShares>> = Map::new("participants_shares");
pub const DRIP_POOLS: Map<String, DripPool> = Map::new("drip_pools");

impl DripToken {
    pub fn into_checked(self, deps: Deps) -> Result<CheckedDripToken, ContractError> {
        match self {
            DripToken::Native {denom, initial_amount} => {
                if initial_amount.is_zero() {
                    Err(ContractError::ZeroTokenPool {})
                }  else {
                    Ok(CheckedDripToken::Native { denom, initial_amount })
                }
            }
            DripToken::CW20 { address, initial_amount } => {
                if initial_amount.is_zero() {
                    Err(ContractError::ZeroTokenPool {})
                } else {
                    let address = deps.api.addr_validate(&address)?;
                    let resp: cw20::TokenInfoResponse = deps.querier.query_wasm_smart(
                        address.clone(), 
                        &cw20::Cw20QueryMsg::TokenInfo {},
                    )?;
                    Ok(CheckedDripToken::CW20 { symbol: resp.symbol, address, initial_amount })
                }
            }
        }
    }
}

impl CheckedDripToken {
    pub fn get_initial_amount(self) -> Uint128 {
        match self {
            CheckedDripToken::Native {denom: _, initial_amount} => {
                return initial_amount
            }
            CheckedDripToken::CW20 { symbol: _, address: _, initial_amount } => {
                return initial_amount
            }
        }
    }
    
    pub fn get_token(self) -> String {
        match self {
            CheckedDripToken::Native {denom, initial_amount: _} => {
                return denom
            }
            // We have to use address because is the only unique
            CheckedDripToken::CW20 { symbol: _, address, initial_amount: _ } => {
                return address.to_string()
            }
        }
    }

    // TODO: find a better way to check and refund excess
    pub fn validate_drip_amount(self, deps: Deps, env: Env) -> Result<(), ContractError> {
        match self {
            Self::Native { denom, initial_amount } => {
                let native_token_balance = deps.querier.query_balance(env.contract.address.to_string(), denom.clone())?;
                if native_token_balance.amount < initial_amount {
                    return Err(ContractError::NoFundedContract { token: denom, amount: initial_amount});
                };
                Ok(())
            },
            Self::CW20 { symbol: _, address, initial_amount } => {
                let cw20_token_balance: cw20::BalanceResponse = deps.querier.query_wasm_smart(
                    address.clone(), 
                    &Cw20QueryMsg::Balance { address: env.contract.address.to_string() }
                )?;
                if cw20_token_balance.balance < initial_amount {
                     return Err(ContractError::NoFundedContract { token: address.to_string(), amount: initial_amount});
                };
                Ok(())
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

    let token = CheckedDripToken::Native { denom: "ujuno".to_string(), initial_amount: Uint128::new(1_000u128) };
    let amount = token.get_initial_amount();
    assert_ne!(amount, Uint128::zero());
    assert_eq!(amount, Uint128::new(1000));
}
