use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, CosmosMsg, StdError, BankMsg, Coin, WasmMsg, to_binary};
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
    /// Epoch number of the last distribution
    pub next_distribution_time: Expiration, 
}

/// Drip pool information saved on storage
#[cw_serde]
pub struct DripPool {
    /// Token to be distributed
    pub drip_token : DripToken,
    /// Initial drip amount 
    pub initial_amount: Uint128,
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
pub enum DripToken {
    Native {
        /// Token denom
        denom: String,
        /// Available tokens
        amount: Uint128,
    },
    CW20 {
        /// CW20 smart contract address
        address: Addr,
        /// Available tokens 
        amount: Uint128,
    }
}

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

impl DripPool {

    /// Given an amount of shares computes the associated tokens and remove both tokens
    /// and shares from the pool.
    pub fn remove_tokens_and_shares(&mut self, shares: Uint128) {
        let tokens = self.tokens_from_shares(shares);
        match self.drip_token {
            DripToken::Native { denom: _, amount } => amount -= tokens,
            DripToken::CW20 { address: _, amount } => amount -= tokens,
        }
        self.issued_shares -= shares

    }

    /// Compute tokens associated to a certain amount of shares
    pub fn tokens_from_shares(&self, shares: Uint128) -> Uint128 {
        let residual_amount: Uint128;
        match self.drip_token {
            DripToken::Native { denom: _, amount } => residual_amount = amount,
            DripToken::CW20 { address: _, amount } => residual_amount = amount,
        }
        let tokens = shares
            .multiply_ratio(residual_amount, self.issued_shares);
        tokens
    }

    pub fn send_tokens_message(self, send_amount: Uint128, recipient: &Addr) -> Result<CosmosMsg, StdError> {
        Ok(match self.drip_token {
            DripToken::Native { denom, amount: _} => BankMsg::Send {
                to_address: recipient.to_string(),
                amount: vec![Coin { denom, amount: send_amount }],
            }
            .into(),
            DripToken::CW20 {
                address,
                amount,
            } => WasmMsg::Execute {
                contract_addr: address.to_string(),
                msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                    recipient: recipient.to_string(),
                    amount: send_amount,
                })?,
                funds: vec![],
            }
            .into(),
        })
    }
}

impl DripToken {
    // Getter for initial amount of the drip token
    pub fn get_available_amount(&self) -> Uint128 {
        match self {
            DripToken::Native {denom: _, amount} => {
                *amount
            }
            DripToken::CW20 { address: _, amount } => {
                *amount
            }
        }
    }
    
    // Getter for the drip token unique identifier
    pub fn get_token(&self) -> String {
        match self {
            DripToken::Native {denom, amount: _} => {
                denom.clone()
            }
            // We have to use address because is the only unique
            DripToken::CW20 { address, amount: _ } => {
                address.to_string().clone()
            }
        }
    }

}