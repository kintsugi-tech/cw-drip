use anyhow::Result as AnyResult;

use cosmwasm_std::{
    Addr, Coin, Validator, FullDelegation, Uint128, Empty, CosmosMsg, StakingMsg, Decimal, 
    Delegation
};
use cw20::{Cw20Coin};
use cw_multi_test::{
    App, AppResponse, Contract, ContractWrapper, Executor, StakingInfo, SudoMsg, BankSudo
};
pub use cw_multi_test::StakeKeeper;

use crate::msg::{
    InstantiateMsg, DripPoolResponse, QueryMsg, DripPoolsResponse, ExecuteMsg, UncheckedDripToken, 
    DripTokensResponse, ParticipantsResponse, ParticipantSharesResponse
};

pub const PAR1: &str = "participant1";
pub const PAR2: &str = "participant2";
pub const PAR3: &str = "participant3";

pub const EPOCH: u64 = 10; // seconds
pub const MIN_STAKING: Uint128 = Uint128::new(1_000_000);

// Contains the initial configuration of the environment.
#[derive(Debug)]
pub struct LabBuilder {
    pub contract_owner: String,
    pub native_token_denom: String,
    pub validators: Vec<String>,
}

pub struct TestLab {
    pub app: App,
    pub owner: String,
    pub native: String,
    pub drip_address: String,
    pub cw20_address: String,

}

// Creates a mock drip contract
pub fn drip_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

// Creates a mock cw20 token contract
pub fn cw20_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

// Helper function to create a Validator structure with default values
fn create_default_validator(validator: &str) -> Validator {
    return Validator {
        address: validator.to_string(),
        commission: Default::default(),
        max_commission: Default::default(),
        max_change_rate: Default::default(),
    }
}

impl LabBuilder {
    
    // Creates a new default environment
    pub fn new() -> Self {
        Self {
            // Owner of the test contracts
            contract_owner: "dao".to_string(),
            // Native staking token denom
            native_token_denom: "ujuno".to_string(),
            validators: vec![
                "validator1".to_string(), "validator2".to_string(), "validator3".to_string()
            ]
        }    
    }

    // Adds to the environment objects and params
    pub fn build(self) -> TestLab {
        // Bootstrapping the mocked blockchain
        let mut app: App = App::default();

        let owner = Addr::unchecked(self.contract_owner.clone());

        let drip_id = app.store_code(drip_contract());

        let init_drip_msg = InstantiateMsg {
            min_staking_amount: MIN_STAKING,
            epoch_duration: EPOCH,
        };

        let drip_addr = app.instantiate_contract (
            drip_id, 
            owner.clone(), 
            &init_drip_msg, 
            &[], 
            "drip", 
            None,
        ).unwrap();

        let block_info = app.block_info();

        // Initialize the environment with funded addresses
        app.init_modules(
            |router, api, storage| -> AnyResult<()> {
                router.staking.setup(
                    storage,
                    StakingInfo { 
                        bonded_denom: self.native_token_denom.clone(), 
                        unbonding_time: 60, 
                        apr: Decimal::percent(20) 
                    }
                ).unwrap();

                self.validators
                    .iter()
                    .for_each(|val| {
                        let validator = create_default_validator(val);
                        router.staking.add_validator(api, storage, &block_info, validator).unwrap();
                    });

            Ok(())
        })
        .unwrap();
   
        TestLab { 
            app,
            owner: owner.to_string(),
            native: self.native_token_denom,
            drip_address: drip_addr.to_string(),
            cw20_address: "None".to_string(), 
        }

    }
    

}

impl TestLab {

    pub fn init_cw20(mut self, initial_balances: Vec<Cw20Coin>) -> Self {

        let cw20_id = self.app.store_code(cw20_contract());

        let init_cw20_msg = &cw20_base::msg::InstantiateMsg {
            name: "stepyt coin".to_string(),
            symbol: "PYT".to_string(),
            initial_balances,
            decimals: 6,
            mint: None,
            marketing: None,
        };

        let cw20_addr = self.app.instantiate_contract(
            cw20_id,
            Addr::unchecked(self.owner.clone()),
            init_cw20_msg,
            &[],
            "coin",
            None,
        )
        .unwrap();

        self.cw20_address = cw20_addr.to_string();
        self
    }

    pub fn advance_blocks(&mut self, added_seconds: u64) {
        self.app.update_block(|block| {
            block.height += added_seconds / 5;
            block.time = block.time.plus_seconds(added_seconds);
        })
    }

    pub fn sudo_mint_1000(mut self, address: String, denom: String, multiplier: u128) -> Self {
        let coin = Coin {
            denom, 
            amount: Uint128::new(1_000 * multiplier)
        };
        self.app.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: address,
            amount: vec![coin],
        }))
        .unwrap();
        self
    }

    pub fn create_delegation(
        &mut self, 
        sender: Addr, 
        validator: String, 
        amount: Coin
    ) -> AppResponse {
        let msg = StakingMsg::Delegate { validator, amount };
        let resp = self.app.execute(sender, CosmosMsg::Staking(msg)).unwrap();
        resp
    }

    pub fn query_participants(& self ) -> ParticipantsResponse {
        let resp: ParticipantsResponse = self.app
            .wrap()
            .query_wasm_smart(self.drip_address.clone(), &QueryMsg::Participants {})
            .unwrap();
        resp
    }

    pub fn query_balance(&self, address: String) -> Uint128 {
        let resp = self.app
            .wrap()
            .query_balance(address, self.native.clone())
            .unwrap();
        resp.amount
    }

    pub fn query_cw20_balance(&self, address: String) -> Uint128 {
        let resp: cw20::BalanceResponse = self.app
            .wrap()
            .query_wasm_smart(
                self.cw20_address.clone(), 
                &cw20::Cw20QueryMsg::Balance { address }
            )
            .unwrap();
        resp.balance
    }

    pub fn get_validators(self) -> Vec<Validator> {
        let resp = self.app
            .wrap()
            .query_all_validators()
            .unwrap();
            resp
    }

    pub fn get_delegations(self, delegator: String) -> Vec<Delegation> {
        let resp = self.app
            .wrap()
            .query_all_delegations(delegator)
            .unwrap();
        resp
    }

    // Returns a specific drip pool 
    pub fn query_drip_pool(&self, token: String) -> DripPoolResponse {
        let resp: DripPoolResponse = self.app
            .wrap()
            .query_wasm_smart(self.drip_address.clone(), &QueryMsg::DripPool {token})
            .unwrap();
        resp
    }

    // Returns all drip pools
    pub fn query_drip_pools(&self) -> DripPoolsResponse {
        let resp: DripPoolsResponse = self.app
            .wrap()
            .query_wasm_smart(self.drip_address.clone(), &QueryMsg::DripPools {  })
            .unwrap();
        resp
    }

    // Returns all drip tokens 
    pub fn query_drip_tokens(&self) -> DripTokensResponse {
        let resp: DripTokensResponse = self.app
            .wrap()
            .query_wasm_smart(self.drip_address.clone(), &QueryMsg::DripTokens {  })
            .unwrap();
        resp
    }

    pub fn query_participant_shares(&self, participant: String) -> ParticipantSharesResponse {
        let resp: ParticipantSharesResponse = self.app
            .wrap()
            .query_wasm_smart(
                self.drip_address.clone(), 
                &QueryMsg::ParticipantShares { address: participant })
            .unwrap();
        resp
    }

    // Create a drip pool
    pub fn create_drip_pool(
        &mut self, 
        token_info: UncheckedDripToken, 
        tokens_per_epoch: Uint128, 
        epochs_number: u64, 
        funds: &[Coin],
    ) -> AnyResult<AppResponse> { 
        self.app.execute_contract(
            Addr::unchecked(self.owner.clone()),
            Addr::unchecked(self.drip_address.clone()),
            &ExecuteMsg::CreateDripPool { 
                token_info,
                tokens_per_epoch,
                epochs_number
            }, 
            funds, 
        )
    }

    pub fn withdraw_tokens(&mut self, address: Addr) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            address, 
            Addr::unchecked(self.drip_address.clone()), 
            &ExecuteMsg::WithdrawTokens {}, 
            &[]
        )
    }

    pub fn distribute_shares(&mut self) -> AnyResult<AppResponse>{
        self.app.execute_contract(
            Addr::unchecked(self.owner.clone()), 
            Addr::unchecked(self.drip_address.clone()), 
            &ExecuteMsg::DistributeShares {}, 
            &[]
        )
    }

    pub fn add_participant(&mut self, participant: Addr) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(participant), 
            Addr::unchecked(self.drip_address.clone()), 
            &ExecuteMsg::Participate {}, 
            &[]
        )
    }

    pub fn remove_participant(&mut self, participant: Addr) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(participant), 
            Addr::unchecked(self.drip_address.clone()), 
            &ExecuteMsg::RemoveParticipation {}, 
            &[],    
        )
    }
   
    fn create_delegator(self, delegator: &str, validator: &str) -> FullDelegation {
        return FullDelegation { 
            delegator: Addr::unchecked(delegator.to_string()), 
            validator: validator.to_string(), 
            amount: Coin { denom: self.native , amount: Uint128::new(1_000_000) }, 
            can_redelegate: Default::default(), 
            accumulated_rewards: Default::default() 
        }
    }

}

