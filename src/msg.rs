use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Addr, Deps, Env};
use cw20::Cw20QueryMsg;
use cw_utils::Duration;

use crate::{state::{Config, DripPool, DripPoolShares, CheckedDripToken}, ContractError};

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of the chain's staking module
    pub staking_module_address: String,
    /// Minimum native tokens staked to participate
    pub min_staking_amount: Uint128,
    /// Duration of a single epoch for all drip pools
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
    // Get participant shares
    #[returns(ParticipantSharesResponse)]
    ParticipantShares {address: String},
}

// Query response structures
#[cw_serde]
pub struct ConfigResponse {
    pub config: Config
}

#[cw_serde]
pub struct ParticipantSharesResponse {
    pub shares: Vec<DripPoolShares>
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

impl DripToken {
    // If Cw20 check if the contract exists.
    pub fn validate_drip_token(&self, deps: Deps) -> Result<(), ContractError> {
        match self {
            DripToken::Native {denom: _, initial_amount: _} => {
               Ok(()) 
            }
            DripToken::CW20 { address, initial_amount: _ } => {
                let address = deps.api.addr_validate(&address)?;
                let _resp: cw20::TokenInfoResponse = deps.querier.query_wasm_smart(
                    address.clone(), 
                    &cw20::Cw20QueryMsg::TokenInfo {},
                )?;
                Ok(())    
            }
        }
    }

    // Check if the smart contract has the required funds for the drip
    // TODO: probably we should set a minimum amount of tokens taht can be distributed
    pub fn validate_drip_amount(self, deps: Deps, env: Env) -> Result<CheckedDripToken, ContractError> {
        match self {
            Self::Native { denom, initial_amount } => {
                if initial_amount.is_zero() {
                    return Err(ContractError::ZeroTokenPool {})
                };
                let native_token_balance = deps.querier.query_balance(env.contract.address.to_string(), denom.clone())?;
                if native_token_balance.amount < initial_amount.clone() {
                    return Err(ContractError::NoFundedContract { token: denom.clone(), amount: initial_amount.clone()});
                };
                Ok(CheckedDripToken::Native { denom, initial_amount })
            },
            Self::CW20 {address, initial_amount } => {
                if initial_amount.is_zero() {
                    return Err(ContractError::ZeroTokenPool {})
                };
                let cw20_token_balance: cw20::BalanceResponse = deps.querier.query_wasm_smart(
                    address.clone(), 
                    &Cw20QueryMsg::Balance { address: env.contract.address.to_string() }
                )?;
                if cw20_token_balance.balance < initial_amount.clone() {
                        return Err(ContractError::NoFundedContract { token: address.to_string(), amount: initial_amount.clone()});
                };
                Ok(CheckedDripToken::CW20 { address, initial_amount })
            }
        }
    }
}