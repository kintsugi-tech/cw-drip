use crate::contract::{instantiate, query};
use crate::msg::{InstantiateMsg, QueryMsg, TotalPowerAtHeightResponse};
use crate::state::Config;
use cosmwasm_std::{Validator, FullDelegation, OwnedDeps, Deps, Env, Addr, from_binary, Uint128};
use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage
};
use cw_utils::Duration;

const DENOM: &str = "ujuno";
const SENDER: &str = "dao";
const STAKING_MODULE: &str = "staking_addr";

const VAL1: &str = "val1";
const VAL2: &str = "val2";
const VAL3: &str = "val3";

fn setup_dependencies(
    validators: Vec<Validator>,
    delegations: Vec<FullDelegation>
) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies();
    deps.querier.update_staking(DENOM, &validators, &delegations);
    deps
}

fn get_config(deps: Deps, env: Env) -> Config {
    let msg = QueryMsg::GetConfig {};
    let bin = query(deps, env, msg).unwrap();
    from_binary(&bin).unwrap()
}

fn get_total_power_at_height(
    deps: Deps,
    env: Env,
    height: Option<u64>,
) -> TotalPowerAtHeightResponse {
    let msg = QueryMsg::TotalPowerAtHeight { height };
    let bin = query(deps, env, msg).unwrap();
    from_binary(&bin).unwrap()
}

fn create_default_validator(name: &str) -> Validator {
    return Validator {
        address: name.to_string(),
        commission: Default::default(),
        max_commission: Default::default(),
        max_change_rate: Default::default()
    }
}

#[test]
fn test_instantiate() {
    let mut deps = setup_dependencies(vec![], vec![]);
    let env = mock_env();
    let info = mock_info(SENDER, &[]);
    let msg = InstantiateMsg {
        staking_module_address: STAKING_MODULE.to_string(),
        epoch_duration: Duration::Height(100)
    };
    let _init_res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    let config = get_config(deps.as_ref(), env.clone());
    assert_eq!(
        config, 
        Config {
            owner: Addr::unchecked(SENDER),
            staking_module_address: Addr::unchecked(STAKING_MODULE),
            epoch_duration: Duration::Height(100)
        }
    )
}

#[test]
fn test_total_power() {
    let mut deps = setup_dependencies(
        vec![
            create_default_validator(VAL1),
            create_default_validator(VAL2),
            create_default_validator(VAL3),
        ],
        vec![]
    );
    let env = mock_env();
    let info = mock_info(SENDER, &[]);
    let msg = InstantiateMsg {
        staking_module_address: STAKING_MODULE.to_string(),
        epoch_duration: Duration::Height(100)
    };
    let _init_res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let power_resp = get_total_power_at_height(deps.as_ref(), env.clone(), None);
    assert_eq!(power_resp.power, Uint128::zero());
    assert_eq!(power_resp.height, env.block.height);
}