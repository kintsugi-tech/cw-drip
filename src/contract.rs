use cosmwasm_std::{entry_point, Addr, StdError, Order, CosmosMsg};
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary, Uint128
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, UncheckedDripToken, ParticipantsResponse, ConfigResponse, 
    DripTokensResponse, DripPoolsResponse, DripPoolResponse, ParticipantSharesResponse
};
use crate::state::{
    Config, CONFIG, PARTICIPANTS_SHARES, DripPool, PARTICIPANTS, DRIP_TOKENS, DRIP_POOLS
};

// Version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-drip";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

//==================================================================================================
// INSTANTIATE 
//==================================================================================================

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;    

    // The contract owner is forced to be the address who send the InstantiateMsg
    // this imposes the instantiation to be performed by the DAO. It will be the only
    // address allowed to create drip pools
    let next_distribution_time = env.block.time.seconds() +  msg.epoch_duration;
    let config = Config {
        owner: info.sender,
        min_staking_amount: msg.min_staking_amount,
        epoch_duration: msg.epoch_duration,
        next_distribution_time, 
    };

    CONFIG.save(deps.storage, &config)?;

    // Initialize other storages to use update on them later
    PARTICIPANTS.save(deps.storage, &Vec::new())?;
    DRIP_TOKENS.save(deps.storage, &Vec::new())?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("next_distribution", next_distribution_time.to_string())
    )
}

//==================================================================================================
// EXECUTE 
//==================================================================================================

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
    ExecuteMsg::SendShares {} => todo!(),
    ExecuteMsg::WithdrawTokens {} => execute_withdraw_tokens(deps, env, info),
   }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Participants {} => to_binary(&query_participants(deps)?),
        QueryMsg::DripTokens {} => to_binary(&query_drip_tokens(deps)?),
        QueryMsg::DripPool { token } => to_binary(&query_drip_pool(deps, token)?),
        QueryMsg::DripPools {} => to_binary(&query_drip_pools(deps)?),
        QueryMsg::ParticipantShares { 
            address 
        } => to_binary(&query_participant_shares(deps, address)?)
    }
}

/// Add the info.sender to the PARTICIPANTS vector or raise an error if it is already inside it
pub fn execute_add_participant(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError>{
    // Add the non participant sender to participants
    PARTICIPANTS.update(deps.storage, |mut participants| {
        if participants.contains(&info.sender) {
            return Err(ContractError::AlreadyParticipant {})
        }
        participants.push(info.sender.clone());
        Ok(participants)
    })?;

    let res = Response::new()
        .add_attribute("action", "add_participant")
        .add_attribute("address", info.sender);
    Ok(res)
}

/// Retains elements of the PARTICIPANTS vector that are different from the info.sender. 
/// No check is made if the info.sender was a participant or not
pub fn execute_remove_participant(
    deps: DepsMut, 
    info: MessageInfo
) -> Result<Response, ContractError> {
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
    token_info: UncheckedDripToken,
    tokens_per_epoch: Uint128,
    epochs_number: u64, 
) -> Result<Response, ContractError> {
    // Only owner can create drip pools
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {})
    };

    // At least on epoch pool
    if epochs_number < 1 {
        return Err(ContractError::LessThanOneEpoch {})
    }

    // Basic checks on token
    let drip_token = token_info.validate(deps.as_ref(), env)?;

    // Required amount for the drip
    let total_drip_amount = tokens_per_epoch
        .checked_mul(epochs_number.into()).map_err(StdError::overflow)?;

    let available_amount = drip_token.get_available_amount();

    if available_amount != total_drip_amount {
        return Err(ContractError::WrongTokensAmount {
            tokens_amount: available_amount, 
            total_tokens: total_drip_amount
        })
    }

    // Check if drip pool exists or create it
    DRIP_POOLS.update(
        deps.storage, 
        drip_token.clone().get_token(), 
        |drip_pool| {
            if drip_pool.is_some() { return Err(ContractError::DripPoolAlreadyExists {}) }
            Ok(DripPool { 
                drip_token: drip_token.clone(),
                initial_amount: drip_token.get_available_amount(), 
                tokens_per_epoch,
                withdrawable_tokens: Uint128::zero(),
                issued_shares: Uint128::zero(), 
                epochs_number, 
                epoch: 0u64
            })
        })?;
    
    // Add token to the list of active pools
    DRIP_TOKENS.update(deps.storage, |mut drip_tokens| -> StdResult<_>{
        drip_tokens.push(drip_token.clone().get_token());
        Ok(drip_tokens)
    })?;

    let res = Response::new()
        .add_attribute("action", "add_drip_pool")
        .add_attribute("token", drip_token.get_token())
        .add_attribute("amount", drip_token.get_available_amount())
        .add_attribute("epochs_number", epochs_number.to_string());
    Ok(res)
}

fn execute_distribute_shares(
    mut deps: DepsMut, 
    env: Env, 
    _info: MessageInfo
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    // At least one pool must be active 
    let drip_tokens = DRIP_TOKENS.load(deps.storage)?;
    if drip_tokens.is_empty() {
        return Err(ContractError::ZeroActiveDripPool {});
    }

    // Initialize shares emitted. Shares emitted will be equal to the sum of the staked tokens
    // of all eligible participants
    let mut emitted_shares = Uint128::zero();

    // Check if pay time!
    if config.next_distribution_time <= env.block.time.seconds() {
        let participants = PARTICIPANTS.load(deps.storage)?;

        for participant in participants {
            let delegations = deps.querier
                .query_all_delegations(participant.clone())?;

            // TODO: add filter to remove micro delegations?
            // Compute total $JUNO delegated by current address
            let total_staked: Uint128 = delegations   
                .iter()
                .map(|delegation| delegation.amount.amount)
                .sum();
            
            if total_staked >= config.min_staking_amount {
                update_participant_shares(&mut deps, &participant, drip_tokens.clone(), total_staked)?;
                emitted_shares += total_staked;
            }
        };

        // Update pools
        let tokens_to_retain = 
            update_drip_pools(&mut deps, drip_tokens, emitted_shares)?;

        // Update drip tokens vector removing expired pool
        DRIP_TOKENS.update(deps.storage, |_| -> StdResult<_> {
            Ok(tokens_to_retain)
        })?;

        // Update new distribution time
        CONFIG.update(deps.storage, |mut config| -> StdResult<_>{
            let time_after_epoch = env.block.time.seconds() - config.next_distribution_time;
            let quotient = time_after_epoch / config.epoch_duration;
            config.next_distribution_time += quotient * config.epoch_duration;
            Ok(config)
        })?;    
    } else {
        return Err(ContractError::NoDistributionTime {})
    }

    let res = Response::new()
        .add_attribute("action", "distribute shares")
        .add_attribute("emitted shares per pool", emitted_shares);
    Ok(res)
}

/// Update the participant active pools shares based on staked amount.
pub fn update_participant_shares(
    deps: &mut DepsMut, 
    participant: &Addr, 
    drip_tokens: Vec<String>, 
    total_staked: Uint128
) -> Result<(), ContractError> {
    for drip_token in drip_tokens {
        PARTICIPANTS_SHARES.update(
            deps.storage, 
            (participant, drip_token),
            |shares| -> StdResult<_> {
            shares.map_or(Ok(total_staked), |mut amount| {
                amount += total_staked;
                Ok(amount)
            })
        })?;
    }
    Ok(())
}

pub fn update_drip_pools(
    deps: &mut DepsMut, 
    drip_tokens: Vec<String>, 
    emitted_shares: Uint128
) -> Result<Vec<String>, ContractError> {
    let mut tokens_to_retain: Vec<String> = vec![];
    // Only token in the drip tokens vector are associated to active pools.
    for drip_token in drip_tokens {
        DRIP_POOLS.update(
            deps.storage, 
            drip_token.clone(), 
            |drip_pool| {
                // Drip pool has been initialized during pool creation so .unwrap() should be ok
                let mut drip_pool = drip_pool.unwrap();

                if drip_pool.epoch > drip_pool.epochs_number {
                    return Err(ContractError::InvalidActiveDripPool {})
                }

                if drip_pool.drip_token.get_available_amount() < drip_pool.tokens_per_epoch {
                    return Err(ContractError::DripPoolHasNotENoughFunds)
                }

                drip_pool.issued_shares += emitted_shares;
                drip_pool.remove_available_tokens(drip_pool.tokens_per_epoch);
                drip_pool.withdrawable_tokens += drip_pool.tokens_per_epoch;
                drip_pool.epoch += 1;

                if drip_pool.epoch < drip_pool.epochs_number {
                    tokens_to_retain.push(drip_token.clone());
                } 

                Ok(drip_pool)
            })?;
    }        
    Ok(tokens_to_retain)     
}

fn execute_withdraw_tokens(
    deps: DepsMut, 
    _env: Env, 
    info: MessageInfo
) -> Result<Response, ContractError> {
    let res: StdResult<Vec<_>> = PARTICIPANTS_SHARES.prefix(&info.sender)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    let participant_shares = res?;

    if participant_shares.is_empty() {
        return Err(ContractError::NoTokensToWithdraw {})
    }

    let mut send_msgs: Vec<CosmosMsg> = vec![];
    for (token, shares) in participant_shares {
        let mut pool = DRIP_POOLS.load(deps.storage, token.clone()).unwrap();
        let withdraw_amount = pool.tokens_from_shares(shares);
        pool.remove_tokens_and_shares(shares);
        send_msgs.push(
            pool.send_tokens_message(withdraw_amount, &info.sender)? 
        );
        DRIP_POOLS.save(deps.storage, token.clone(), &pool)?;
        PARTICIPANTS_SHARES.remove(deps.storage, (&info.sender, token));
    }

    
    let res = Response::new()
        .add_attribute("action", "withdraw_tokens")
        .add_messages(send_msgs);
    Ok(res) 
}

//==================================================================================================
// QUERY 
//==================================================================================================

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
            let drip_pool = DRIP_POOLS.load(deps.storage, token)?;
            Ok(drip_pool)
        })
        .collect::<StdResult<Vec<DripPool>>>()?;
    Ok(DripPoolsResponse {
        drip_pools
    })
}

fn query_participant_shares(deps: Deps, address: String) -> StdResult<ParticipantSharesResponse> {
    let address = &deps.api.addr_validate(&address)?;
    let resp: StdResult<Vec<_>> = PARTICIPANTS_SHARES
        .prefix(address)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    let shares = resp?;
    Ok(ParticipantSharesResponse { shares } )
}