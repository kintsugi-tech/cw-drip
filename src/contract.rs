use cosmwasm_std::{entry_point, Addr};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary, Uint128};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TotalPowerAtHeightResponse, VotingPowerAtHeightReponse, AddParticipantResponse, TokenInfo};
use crate::state::{Config, CONFIG, PARTICIPANTS_SHARES, DripPool, CheckedTokenInfo, PARTICIPANTS, DRIP_TOKENS};


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

    // The contract owner is always the address who send the InstantiateMsg
    let config = Config {
        owner: info.sender,
        staking_module_address: staking_address,
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
    ExecuteMsg::CreateDripPool { 
        token_info, 
        min_staking_amount, 
        epochs_number 
    } => execute_create_drip_pool(deps, info, token_info, min_staking_amount, epochs_number)
   }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => {to_binary(&query_config(deps)?)}
        QueryMsg::Participants {} => {to_binary(&query_participants(deps)?)}
        QueryMsg::DripTokens {} => {to_binary(&query_drip_tokens(deps)?)}
        QueryMsg::TotalPowerAtHeight {height} => {
            to_binary(&query_total_power_at_height(deps, env, height)?)
        }
        QueryMsg::VotingPowerAtHeight {address, height} => todo!()
    }
}

fn query_drip_tokens(deps: Deps) -> StdResult<Vec<String>> {
    let drip_tokens = DRIP_TOKENS.load(deps.storage)?;
    Ok(drip_tokens)
}

fn query_participants(deps: Deps) -> StdResult<Vec<Addr>> {
    let participants = PARTICIPANTS.load(deps.storage)?;
    Ok(participants)
}

pub fn execute_add_participant(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError>{
    
    let staking_denom = deps.querier.query_bonded_denom()?;

    if PARTICIPANTS_SHARES.has(deps.storage, (staking_denom.clone(), &info.sender)) {
        return Err(ContractError::AlreadyParticipant {});
    };

    PARTICIPANTS.update(deps.storage, |participants| -> StdResult<_> {
        Ok(participants)
        }   
    );
    // Initialize participant with zero shares of the staking token.
    PARTICIPANTS_SHARES.save(deps.storage, (staking_denom.clone(), &info.sender), &Uint128::zero())?;

    let res = Response::new()
        .add_attribute("action", "add_participant")
        .add_attribute("addess", info.sender)
        .add_attribute("denom", staking_denom)
        .add_attribute("shares", Uint128::zero().to_string());
    Ok(res)


}

pub fn execute_create_drip_pool(
    deps: DepsMut,
    info: MessageInfo,
    token_info: TokenInfo,
    min_staking_amount: Uint128,
    epochs_number: u64, 
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {})
    };

    let drip_token = token_info.into_checked(deps.as_ref())?;
    let drip_pool = DripPool {
       drip_token: drip_token.clone(),
       actual_amount: drip_token.get_initial_amount(),
       issued_shares: Uint128::zero(),
       min_staking_amount,
       epochs_number,
       current_epoch: 0u64,
    };

     
    
    todo!()
}

pub fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

pub fn query_total_power_at_height(
    deps: Deps,
    env: Env,
    height: Option<u64>
) -> StdResult<TotalPowerAtHeightResponse> {
    let config = query_config(deps)?;
    let denom = deps.querier.query_bonded_denom()?;
    let power = deps.querier.query_balance(config.staking_module_address, denom)?;
    Ok(TotalPowerAtHeightResponse { 
        power: power.amount, 
        height: height.unwrap_or(env.block.height)}) 
}

