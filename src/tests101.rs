use crate::ContractError;
use crate::contract::{instantiate, query};
use crate::msg::{InstantiateMsg, QueryMsg, TotalPowerAtHeightResponse, ExecuteMsg, ParticipantsResponse, DripPoolsResponse, DripToken, DripTokensResponse, DripPoolResponse, ConfigResponse};
use crate::state::{Config, CheckedDripToken, DripPool};
use cosmwasm_std::{Empty, Coin};
use cosmwasm_std::{
    Validator, FullDelegation, OwnedDeps, Deps, Env, Addr, from_binary, Uint128,
    testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage}
    };
use cw20::Cw20Coin;
use cw_multi_test::{Contract, ContractWrapper, App, Executor, SudoMsg, BankSudo};
use cw_utils::{Duration, Expiration};

const DENOM: &str = "ujuno";
const SENDER: &str = "dao";
const STAKING_MODULE: &str = "staking_addr";

const VAL1: &str = "val1";
const VAL2: &str = "val2";
const VAL3: &str = "val3";

const PAR1: &str = "participant1";
const PAR2: &str = "participant2";
const PAR3: &str = "participant3";

// Create a mock drip contract
fn drip_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

// Create a mock cw20 token contract
fn cw20_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

// Helper function to easily mock staking module
fn setup_dependencies(
    validators: Vec<Validator>,
    delegations: Vec<FullDelegation>
) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies();
    deps.querier.update_staking(DENOM, &validators, &delegations);
    deps
}

fn get_config(deps: Deps, env: Env) -> ConfigResponse {
    let msg = QueryMsg::Config {};
    let bin = query(deps, env, msg).unwrap();
    from_binary(&bin).unwrap()
}


fn get_participants(deps: Deps, env: Env) -> ParticipantsResponse {
    let msg = QueryMsg::Participants {};
    let bin = query(deps, env, msg).unwrap();
    from_binary(&bin).unwrap()
}

fn get_drip_tokens(deps: Deps, env: Env) -> DripTokensResponse {
    let msg = QueryMsg::DripTokens {};
    let bin = query(deps, env, msg).unwrap();
    from_binary(&bin).unwrap()
}

// Helper function to create a Validator structure
fn create_default_validator(name: &str) -> Validator {
    return Validator {
        address: name.to_string(),
        commission: Default::default(),
        max_commission: Default::default(),
        max_change_rate: Default::default()
    }
}

// Query for getting the vector of drip tokens
fn query_drip_tokens (app: &App, drip_contract_addr: Addr) -> DripTokensResponse {
    let resp: DripTokensResponse= app
        .wrap()
        .query_wasm_smart(drip_contract_addr, &QueryMsg::DripTokens {})
        .unwrap();
    resp
}

// Query for getting info for a specific pool 
fn query_drip_pool(app: &App, drip_contract_addr: Addr, token: String) -> DripPoolResponse {
    let resp: DripPoolResponse = app
        .wrap()
        .query_wasm_smart(drip_contract_addr.clone(), &QueryMsg::DripPool {token})
        .unwrap();
    resp
}

// Query for getting all the drip pools
fn query_drip_pools(app: &App, drip_contract_addr: Addr) -> DripPoolsResponse {
    let resp: DripPoolsResponse = app
        .wrap()
        .query_wasm_smart(drip_contract_addr.clone(), &QueryMsg::DripPools {})
        .unwrap();
    resp
}


#[test]
fn test_instantiate() {
    let mut deps = setup_dependencies(vec![], vec![]);
    let env = mock_env();
    let info = mock_info(SENDER, &[]);
    let msg = InstantiateMsg {
        staking_module_address: STAKING_MODULE.to_string(),
        min_staking_amount: Uint128::zero(),
        epoch_duration: Duration::Height(100)
    };
    let _init_res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    let config = get_config(deps.as_ref(), env.clone());
    
    assert_eq!(
        config.config, 
        Config {
            owner: Addr::unchecked(SENDER),
            staking_module_address: Addr::unchecked(STAKING_MODULE),
            min_staking_amount: Uint128::zero(),
            epoch_duration: Duration::Height(100),
            last_distribution_time: None,
            next_distribution_time: Expiration::AtHeight(env.block.height + 100),
        }
    );

    let participants = get_participants(deps.as_ref(), env.clone());
    let no_participants: Vec<Addr> = Vec::new();
    assert_eq!(participants, ParticipantsResponse { participants: no_participants});

    let drip_tokens = get_drip_tokens(deps.as_ref(), env.clone());
    let no_drip_tokens: Vec<String> = Vec::new();
    assert_eq!(drip_tokens, DripTokensResponse { drip_tokens: no_drip_tokens});


}

#[test]
fn test_add_participant() {
    let mut app = App::default();

    let drip_contract_code_id = app.store_code(drip_contract());

    let instantiate_msg = InstantiateMsg {
        staking_module_address: STAKING_MODULE.to_string(),
        min_staking_amount: Uint128::zero(),
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
   
    // Check empty participants vector before any execution
    let resp: ParticipantsResponse = app
        .wrap()
        .query_wasm_smart(drip_contract_addr.clone(), &QueryMsg::Participants {})
        .unwrap();

    assert_eq!(resp.participants.len(), 0);

    // Add a participant and check for correct vector update
    app.execute_contract(
        Addr::unchecked(PAR1), 
        drip_contract_addr.clone(), 
        &ExecuteMsg::Participate {}, 
        &[]
    )
    .unwrap();
   
    let resp: ParticipantsResponse = app
        .wrap()
        .query_wasm_smart(drip_contract_addr, &QueryMsg::Participants {})
        .unwrap();

    assert_eq!(resp.participants.len(), 1);
    assert_eq!(
        resp.participants,
        vec![Addr::unchecked(PAR1)]
    )

}

#[test]
fn test_remove_participant() {
    let mut app = App::default();

    let drip_contract_code_id = app.store_code(drip_contract());

    let instantiate_msg = InstantiateMsg {
        staking_module_address: STAKING_MODULE.to_string(),
        min_staking_amount: Uint128::zero(),
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
   
    // Check empty participants vector before any execution
    let resp: ParticipantsResponse = app
        .wrap()
        .query_wasm_smart(drip_contract_addr.clone(), &QueryMsg::Participants {})
        .unwrap();

    assert_eq!(resp.participants.len(), 0);

    // Add a participant and check for correct vector update
    app.execute_contract(
        Addr::unchecked(PAR1), 
        drip_contract_addr.clone(), 
        &ExecuteMsg::Participate {}, 
        &[]
    )
    .unwrap();
   
    app.execute_contract(
        Addr::unchecked(SENDER), 
        drip_contract_addr.clone(),
        &ExecuteMsg::RemoveParticipation {}, 
        &[]
    )
    .unwrap();

    let resp: ParticipantsResponse = app
        .wrap()
        .query_wasm_smart(drip_contract_addr.clone(), &QueryMsg::Participants {})
        .unwrap();

    assert_eq!(resp.participants.len(), 1);
    
    app.execute_contract(
        Addr::unchecked(PAR1), 
        drip_contract_addr.clone(),
        &ExecuteMsg::RemoveParticipation {}, 
        &[]
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked(PAR2), 
        drip_contract_addr.clone(), 
        &ExecuteMsg::Participate {}, 
        &[]
    )
    .unwrap();
 

    let resp: ParticipantsResponse = app
        .wrap()
        .query_wasm_smart(drip_contract_addr, &QueryMsg::Participants {})
        .unwrap();

    assert_eq!(resp.participants.len(), 1);
    assert_eq!(
        resp.participants,
        vec![Addr::unchecked(PAR2)]
    )

}