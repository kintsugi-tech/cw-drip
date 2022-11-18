#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TotalPowerAtHeightResponse, VotingPowerAtHeightReponse};
use crate::state::{Config, CONFIG};


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

    let config = Config {
        owner: info.sender,
        staking_module_address: staking_address,
        epoch_duration: msg.epoch_duration
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
   match msg {}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::TotalPowerAtHeight {height} => {
            to_binary(&query_total_power_at_height(deps, env, height)?)
        }
        QueryMsg::VotingPowerAtHeight {address, height} => todo!()
    }
}

pub fn query_total_power_at_height(
    deps: Deps,
    env: Env,
    height: Option<u64>
) -> StdResult<TotalPowerAtHeightResponse> {
    let config = CONFIG.load(deps.storage)?;
    let denom = deps.querier.query_bonded_denom()?;
    let power = deps.querier.query_balance(config.staking_module_address, denom)?;
    Ok(TotalPowerAtHeightResponse { 
        power: power.amount, 
        height: height.unwrap_or(env.block.height)}) 
}


#[cfg(test)]
mod tests {}
