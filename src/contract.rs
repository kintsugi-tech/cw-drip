use cosmwasm_std::{entry_point, Addr};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary, Uint128};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, DripToken, ParticipantsResponse, ConfigResponse, DripTokensResponse, DripPoolsResponse, DripPoolResponse, ParticipantSharesResponse};
use crate::state::{Config, CONFIG, PARTICIPANTS_SHARES, DripPool, PARTICIPANTS, DRIP_TOKENS, DRIP_POOLS, DripPoolShares};


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-drip";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;    

    let staking_address = deps.api.addr_validate(&msg.staking_module_address)?;

    // The contract owner is forced to be the address who send the InstantiateMsg
    // this imposes the instantiation to be performed by the DAO. It will be the only
    // address allowed to create drip pools.
    let config = Config {
        owner: info.sender,
        staking_module_address: staking_address,
        min_staking_amount: msg.min_staking_amount,
        epoch_duration: msg.epoch_duration.clone(),
        last_distribution_time: None,
        next_distribution_time: msg.epoch_duration.after(&env.block), 
    };

    CONFIG.save(deps.storage, &config)?;
    // Initialize other storages to use update on them later
    PARTICIPANTS.save(deps.storage, &Vec::new())?;
    DRIP_TOKENS.save(deps.storage, &Vec::new())?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
   match msg {
    ExecuteMsg::Participate {} => execute_add_participant(deps, info),
    ExecuteMsg::RemoveParticipation {} => execute_remove_participant(deps, info), 
    ExecuteMsg::CreateDripPool { 
        token_info,
        tokens_per_epoch, 
        epochs_number 
    } => execute_create_drip_pool(deps, env, info, token_info,tokens_per_epoch, epochs_number),
    ExecuteMsg::UpdateDripPool {} => todo!(),
    ExecuteMsg::RemoveDripPool {} => todo!(),
    ExecuteMsg::DistributeShares {} => execute_distribute_shares(deps, env, info),
    ExecuteMsg::WithdrawToken {  } => todo!(),
    ExecuteMsg::WithdrawTokens {  } => todo!(),
   }
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Participants {} => to_binary(&query_participants(deps)?),
        QueryMsg::DripTokens {} => to_binary(&query_drip_tokens(deps)?),
        QueryMsg::DripPool { token } => to_binary(&query_drip_pool(deps, token)?),
        QueryMsg::DripPools {} => to_binary(&query_drip_pools(deps)?),
        QueryMsg::ParticipantShares { address } => to_binary(&query_participant_shares(deps, address)?)
    }
}


pub fn execute_add_participant(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError>{
    
    // TODO: check if faster than looking at PARTICIPANTS
    if PARTICIPANTS_SHARES.has(deps.storage, &info.sender) {
        return Err(ContractError::AlreadyParticipant {});
    };

    // Add the sender as a participant
    PARTICIPANTS.update(deps.storage, |mut participants| -> StdResult<_> {
        participants.push(info.sender.clone());
        Ok(participants)
    })?;

    // Initialize participant vector of shares 
    PARTICIPANTS_SHARES.save(deps.storage, &info.sender, &Vec::new())?;

    let res = Response::new()
        .add_attribute("action", "add_participant")
        .add_attribute("addess", info.sender);
    Ok(res)
}

fn execute_remove_participant(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    PARTICIPANTS.update(deps.storage, |mut participants| -> StdResult<_>{
        participants.retain(|address| *address != info.sender);
        Ok(participants)
    })?;

    let res = Response::new()
       .add_attribute("action", "remove_participant")
       .add_attribute("address", info.sender.to_string());
    Ok(res)
}

pub fn execute_create_drip_pool(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_info: DripToken,
    tokens_per_epoch: Uint128,
    epochs_number: u64, 
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {})
    };

    // Basic checks on token_info
    token_info.validate_drip_token(deps.as_ref())?;
    let drip_token = token_info.validate_drip_amount(deps.as_ref(), env)?;

    let total_drip_amount = tokens_per_epoch.checked_mul(epochs_number.into()).unwrap();

    if drip_token.clone().get_initial_amount() != total_drip_amount {
        return Err(ContractError::WrongTokensAmount {tokens_amount: drip_token.get_initial_amount(), total_tokens: total_drip_amount})
    }

    // TODO: check on epochs number > 0
    // TODO: add an upper bound for epochs number? It is meaningless allowing 0 tokens_per_epochs

    // Check if drip pool already exists or create it
    DRIP_POOLS.update(deps.storage, drip_token.clone().get_token(), |drip_pool| -> Result<DripPool, ContractError> {
        match drip_pool {
            Some(_)  => Err(ContractError::DripPoolAlreadyExists {}),
            None => {
                Ok(
                    DripPool { 
                        drip_token: drip_token.clone(), 
                        actual_amount: total_drip_amount,
                        tokens_to_withdraw: Uint128::zero(),
                        tokens_per_epoch,
                        issued_shares: Uint128::zero(), 
                        epochs_number, 
                        current_epoch: 0u64
                    }
                )
            }
        } 
    })?;
    
    
    // Save the drip token denom or address into storage
    DRIP_TOKENS.update(deps.storage, |mut drip_tokens| -> StdResult<_>{
        drip_tokens.push(drip_token.clone().get_token());
        Ok(drip_tokens)
    })?;

    let res = Response::new()
        .add_attribute("action", "add_drip_pool")
        .add_attribute("token", drip_token.clone().get_token())
        .add_attribute("amount", drip_token.get_initial_amount())
        .add_attribute("epochs_number", epochs_number.to_string());
    Ok(res)
}

/// Message handler for the distribution of the active drip pools' shares to eligible participants.
fn execute_distribute_shares(mut deps: DepsMut, env: Env, _info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    // Check if there is at least one drip pool to distribute shares
    // TODO: maybe we can remove this error. Pools are removed the distribution after the last onebut this can be e
    // easily changed.
    let drip_tokens = DRIP_TOKENS.load(deps.storage)?;
    if drip_tokens.is_empty() {
        return Err(ContractError::ZeroActiveDripPool {});
    }

    // Initialize shares emitted. Shares emitted will be equal to the sum of the staked tokens
    // of all eligible participants
    let mut emitted_shares = Uint128::zero();

    // Check if pay time!
    if config.next_distribution_time.is_expired(&env.block) {
        let participants = PARTICIPANTS.load(deps.storage)?;

        for participant in participants {
            // Query all delegation of the current participant
            let delegations = deps.querier.query_all_delegations(participant.clone())?;

            // TODO: add filter to remove micro delegations?
            // Compute total $JUNO delegated by current address
            let total_staked: Uint128 = delegations   
                .iter()
                .map(|delegation| delegation.amount.amount)
                .sum();
            
            // If eligible then distribute shares
            if total_staked >= config.min_staking_amount {
                // Shares are distribute per drip pool
                update_participant_shares(&mut deps, &participant, drip_tokens.clone(), total_staked)?;
                emitted_shares += total_staked;
            }
        };

        // Update pools
        let tokens_to_retain = update_drip_pools(& mut deps, drip_tokens, emitted_shares)?;

        DRIP_TOKENS.update(deps.storage, |_| -> StdResult<_> {
            Ok(tokens_to_retain)
        })?;
        
    } else {
        // TODO: add info on next distribution time
        return Err(ContractError::NoDistributionTime {})
    }

    let res = Response::new()
        .add_attribute("action", "distribute shares")
        .add_attribute("emitted shares per pool", emitted_shares);
    Ok(res)
}

/// Update the participant active pools shares based on staked amount.
pub fn update_participant_shares<'a>(deps: &'a mut DepsMut, participant: &Addr, drip_tokens: Vec<String>, total_staked: Uint128) -> Result<(), ContractError> {
    PARTICIPANTS_SHARES.update(deps.storage, participant, |shares| -> StdResult<_> {
        let  mut shares = shares.unwrap_or_default();
        for drip_token in drip_tokens {
            let pool_position = shares.iter().position(|pool| pool.token == drip_token.clone());
            // If position not found it is the first distribution so add to vector,
            // otherwise update
            match pool_position {
                Some(position) => shares[position].total_shares += total_staked,
                None => shares.push(DripPoolShares {
                    token: drip_token.clone(),
                    total_shares: total_staked,
                })
            };
        }
        Ok(shares)
    })?;
    Ok(())
}

/// Update the active drip pools by emitting shares and removing the distributed tokens from
/// availability.
pub fn update_drip_pools<'a>(deps: &'a mut DepsMut, drip_tokens: Vec<String>, emitted_shares: Uint128) -> Result<Vec<String>, ContractError> {
    let mut tokens_to_retain: Vec<String> = vec![];
    // Only token in the drip tokens vector are associated to active pools.
    for drip_token in drip_tokens {
        DRIP_POOLS.update(deps.storage, drip_token.clone(), |drip_pool| -> StdResult<_> {
            // Drip pool has been initialized during pool creation so .unwrap() should be ok.
            let mut drip_pool = drip_pool.unwrap();
            // If drip pool is valid distribute, else remove drip pool.
            if drip_pool.current_epoch <= drip_pool.epochs_number && drip_pool.actual_amount >= drip_pool.tokens_per_epoch {
                drip_pool.issued_shares += emitted_shares;
                drip_pool.actual_amount -= drip_pool.tokens_per_epoch;
                drip_pool.tokens_to_withdraw += drip_pool.tokens_per_epoch;
                drip_pool.current_epoch += 1;
                tokens_to_retain.push(drip_token.clone())
            };

            Ok(drip_pool)
        })?;
    }        
    Ok(tokens_to_retain)     
}


pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    Ok(ConfigResponse {
        config: CONFIG.load(deps.storage)?
    })
}

fn query_drip_tokens(deps: Deps) -> StdResult<DripTokensResponse> {
    Ok(DripTokensResponse {
        drip_tokens: DRIP_TOKENS.load(deps.storage)?
    })
}

fn query_participants(deps: Deps) -> StdResult<ParticipantsResponse> {
    Ok(ParticipantsResponse {
        participants: PARTICIPANTS.load(deps.storage)?
    })
}

fn query_drip_pool(deps: Deps, token: String) -> StdResult<DripPoolResponse> {
    let drip_pool = DRIP_POOLS.may_load(deps.storage, token)?;
    match drip_pool {
        Some(drip_pool) => Ok(DripPoolResponse {
            drip_pool: Some(drip_pool)
        }),
        None => Ok(DripPoolResponse { drip_pool: None })
    }
}

fn query_drip_pools(deps: Deps) -> StdResult<DripPoolsResponse> {
    let drip_tokens = DRIP_TOKENS.load(deps.storage)?;
    let drip_pools = drip_tokens
        .into_iter()
        .map(|token| {
            let drip_pool = DRIP_POOLS.load(deps.storage, token.clone())?;
            Ok(drip_pool)
        })
        .collect::<StdResult<Vec<DripPool>>>()?;
    Ok(DripPoolsResponse {
        drip_pools
    })
}

fn query_participant_shares(deps: Deps, address: String) -> StdResult<ParticipantSharesResponse> {
    let address = &deps.api.addr_validate(&address)?;
    let participant_shares = PARTICIPANTS_SHARES.may_load(deps.storage, address)?;
    match participant_shares {
        Some(shares) => return Ok(ParticipantSharesResponse{ shares }),
        None => return Ok(ParticipantSharesResponse { shares: vec![] }),
    };
}