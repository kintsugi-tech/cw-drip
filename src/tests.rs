use crate::contract::{instantiate, query};
use crate::msg::{InstantiateMsg, QueryMsg, TotalPowerAtHeightResponse, ExecuteMsg, ConfigResponse};
use crate::state::{Config, PARTICIPANTS};
use cosmwasm_std::Empty;
use cosmwasm_std::{
    Validator, FullDelegation, OwnedDeps, Deps, Env, Addr, from_binary, Uint128,
    testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage}
    };
use cw_multi_test::{Contract, ContractWrapper, App, Executor};
use cw_utils::Duration;

const DENOM: &str = "ujuno";
const SENDER: &str = "dao";
const STAKING_MODULE: &str = "staking_addr";

const VAL1: &str = "val1";
const VAL2: &str = "val2";
const VAL3: &str = "val3";

const PAR1: &str ="participant1";
const PAR2: &str ="participant2";
const PAR3: &str ="participant3";


fn drip_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

fn cw20_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

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


fn get_participants(deps: Deps, env: Env) -> Vec<Addr> {
    let msg = QueryMsg::Participants {};
    let bin = query(deps, env, msg).unwrap();
    from_binary(&bin).unwrap()
}

fn get_drip_tokens(deps: Deps, env: Env) -> Vec<String> {
    let msg = QueryMsg::DripTokens {};
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
    );

    let participants = get_participants(deps.as_ref(), env.clone());
    let no_participants: Vec<Addr> = Vec::new();
    assert_eq!(participants, no_participants);

    let drip_tokens = get_participants(deps.as_ref(), env.clone());
    let no_drip_tokens: Vec<String> = Vec::new();
    assert_eq!(drip_tokens, no_drip_tokens);


}

#[test]
fn test_add_participant() {
    let mut app = App::default();

    let drip_contract_code_id = app.store_code(drip_contract());

    let instantiate_msg = InstantiateMsg {
        staking_module_address: STAKING_MODULE.to_string(),
        epoch_duration: Duration::Height(100)
    };

    let drip_contract_addr = app.instantiate_contract(
        drip_contract_code_id, 
        Addr::unchecked(SENDER), 
        &instantiate_msg, 
        &[], 
        "drip", 
        None,
    ).unwrap();

    app.execute_contract(
        Addr::unchecked(SENDER), 
        drip_contract_addr.clone(), 
        &ExecuteMsg::Participate {}, 
        &[]
    )
    .unwrap();

    participants: ConfigResponse = app
        .wrap()
        .query_wasm_smart(drip_contract_addr, &QueryMsg::GetConfig {  })
        .unwrap()

        

}

#[test]
fn test_total_power_at_height() {
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