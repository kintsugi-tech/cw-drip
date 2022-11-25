use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Deps, MessageInfo, Env};
use cw_storage_plus::{Item, Map};
use cw_utils::Duration;

use crate::{ContractError, msg::TokenInfo};

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
    pub epoch_duration: Duration
}

#[cw_serde]
pub struct DripPool {
    pub drip_token : CheckedTokenInfo,
    pub actual_amount: Uint128,
    pub issued_shares: Uint128,
    pub epochs_number: u64,
    pub current_epoch: u64,
}

#[cw_serde]
pub enum CheckedTokenInfo {
    Native {
        denom: String,
        initial_amount: Uint128,
    },
    CW20 {
        symbol: String,
        address: Addr,
        initial_amount: Uint128,
    }
}

#[cw_serde]
pub struct DripPoolShares {
    // Denom or address of the token
    token: String,
    // Total amount of shares
    totale_shares: Uint128
}

pub const CONFIG: Item<Config> = Item::new("config");

// Storage for the list of all participants.
pub const PARTICIPANTS: Item<Vec<Addr>> = Item::new("participants");

// Storage for the list of all drip tokens.
pub const DRIP_TOKENS: Item<Vec<String>> = Item::new("drip_tokens");

pub const PARTICIPANTS_SHARES: Map<&Addr, Vec<DripPoolShares>> = Map::new("participants_shares");
pub const DRIP_POOLS: Map<String, DripPool> = Map::new("drip_pools");

impl TokenInfo {
    pub fn into_checked(self, deps: Deps) -> Result<CheckedTokenInfo, ContractError> {
        match self {
            TokenInfo::Native {denom, initial_amount} => {
                if initial_amount.is_zero() {
                    Err(ContractError::ZeroTokenPool {})
                }  else {
                    Ok(CheckedTokenInfo::Native { denom, initial_amount })
                }
            }
            TokenInfo::CW20 { address, initial_amount } => {
                if initial_amount.is_zero() {
                    Err(ContractError::ZeroTokenPool {})
                } else {
                    let address = deps.api.addr_validate(&address)?;
                    let resp: cw20::TokenInfoResponse = deps.querier.query_wasm_smart(
                        address.clone(), 
                        &cw20::Cw20QueryMsg::TokenInfo {},
                    )?;
                    Ok(CheckedTokenInfo::CW20 { symbol: resp.symbol, address, initial_amount })
                }
            }
        }
    }
}

impl CheckedTokenInfo {
    pub fn get_initial_amount(self) -> Uint128 {
        match self {
            CheckedTokenInfo::Native {denom: _, initial_amount} => {
                return initial_amount
            }
            CheckedTokenInfo::CW20 { symbol: _, address: _, initial_amount } => {
                return initial_amount
            }
        }
    }
    
    pub fn get_token(self) -> String {
        match self {
            CheckedTokenInfo::Native {denom, initial_amount: _} => {
                return denom
            }
            CheckedTokenInfo::CW20 { symbol: _, address, initial_amount: _ } => {
                return address.to_string()
            }
        }
    }

    // TODO: find a better way to check and refund excess
    pub fn check_balance(self, info: MessageInfo, deps: Deps, env: Env) -> Result<bool, ContractError> {
        match self {
            CheckedTokenInfo::Native { denom: _, initial_amount } => {
                // TODO: generalize 
                if initial_amount != info.funds[0].amount {
                    Err(ContractError::NoFundedContract {})
                } else {
                    Ok(true)
                } 
            }
            CheckedTokenInfo::CW20 { symbol: _, address, initial_amount } => {
                let contract_funds: cw20::BalanceResponse = deps.querier.query_wasm_smart(
                    address, 
                    &cw20::Cw20QueryMsg::Balance {address: env.contract.address.into()},
                )?;
                if initial_amount != contract_funds.balance {
                    Err(ContractError::NoFundedContract {})
                } else {
                    Ok(true)
                }
            }
        }
    }
}

#[cfg(test)]
#[test]
fn test_get_initial_amount() {
    let token = CheckedTokenInfo::Native { denom: "ujuno".to_string(), initial_amount: Uint128::zero() };
    let amount = token.get_initial_amount();
    assert_eq!(amount, Uint128::zero());

    let token = CheckedTokenInfo::Native { denom: "ujuno".to_string(), initial_amount: Uint128::new(1_000u128) };
    let amount = token.get_initial_amount();
    assert_ne!(amount, Uint128::zero());
    assert_eq!(amount, Uint128::new(1000));
}
