use cosmwasm_std::{entry_point};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary, Uint128};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TotalPowerAtHeightResponse, DripToken, ParticipantsResponse, ConfigResponse, DripTokensResponse, DripPoolsResponse, DripPoolResponse};
use crate::state::{Config, CONFIG, PARTICIPANTS_SHARES, DripPool, PARTICIPANTS, DRIP_TOKENS, DRIP_POOLS};


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-drip";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;    

    let staking_address = deps.api.addr_validate(&msg.staking_module_address)?;

    // The contract owner is forced to be the address who send the InstantiateMsg
    let config = Config {
        owner: info.sender,
        staking_module_address: staking_address,
        min_staking_amount: msg.min_staking_amount,
        epoch_duration: msg.epoch_duration
    };

    CONFIG.save(deps.storage, &config)?;
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
    ExecuteMsg::RemoveParticipation {  } => execute_remove_participant(deps, info), 
    ExecuteMsg::CreateDripPool { 
        token_info, 
        epochs_number 
    } => execute_create_drip_pool(deps, env, info, token_info, epochs_number),
    ExecuteMsg::UpdateDripPool {} => todo!(),
    ExecuteMsg::RemoveDripPool {} => todo!(),
    ExecuteMsg::DistributeShares {  } => todo!(),
    ExecuteMsg::WithdrawToken {  } => todo!(),
    ExecuteMsg::WithdrawTokens {  } => todo!(),
   }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {to_binary(&query_config(deps)?)}
        QueryMsg::Participants {} => {to_binary(&query_participants(deps)?)}
        QueryMsg::DripTokens {} => {to_binary(&query_drip_tokens(deps)?)}
        QueryMsg::DripPool {token} => {to_binary(&query_drip_pool(deps, token)?)}
        QueryMsg::DripPools {} => {to_binary(&query_drip_pools(deps)?)}
        QueryMsg::TotalPowerAtHeight {height} => {
            to_binary(&query_total_power_at_height(deps, env, height)?)
        }
        QueryMsg::VotingPowerAtHeight {address, height} => todo!()
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
    epochs_number: u64, 
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {})
    };

    // Basic checks on token_info
    let drip_token = token_info.into_checked(deps.as_ref())?;

    // Check if drip pool already exists or create it
    DRIP_POOLS.update(deps.storage, drip_token.clone().get_token(), |drip_pool| -> Result<DripPool, ContractError> {
        match drip_pool {
            Some(_)  => Err(ContractError::DripPoolAlreadyExists {}),
            None => {
                Ok(
                    DripPool { 
                        drip_token: drip_token.clone(), 
                        actual_amount: drip_token.clone().get_initial_amount(), 
                        issued_shares: Uint128::zero(), 
                        epochs_number, 
                        current_epoch: 0u64
                    }
                )
            }
        } 
    })?;
    
    // TODO: how to handle it without clone()
    drip_token.clone().validate_drip_amount(deps.as_ref(), env)?;

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

pub fn query_total_power_at_height(
    deps: Deps,
    env: Env,
    height: Option<u64>
) -> StdResult<TotalPowerAtHeightResponse> {
    let config_resp = query_config(deps)?;
    let denom = deps.querier.query_bonded_denom()?;
    let power = deps.querier.query_balance(config_resp.config.staking_module_address, denom)?;
    Ok(TotalPowerAtHeightResponse { 
        power: power.amount, 
        height: height.unwrap_or(env.block.height)}) 
}

