use cosmwasm_std::{Uint128};
use cw20::Cw20Coin;

use crate::{msg::UncheckedDripToken, ContractError, state::{DripPool, DripToken}};

use super::environment::LabBuilder;

#[test]
fn zero_initial_amount() {
    let mut test_lab = LabBuilder::new().build();
    let initial_balances = vec![
        Cw20Coin {
            address: test_lab.owner.to_string(),
            amount: Uint128::new(1_000_000)
        }];
    test_lab = test_lab.init_cw20(initial_balances);

    // With native pool
    let err: ContractError = test_lab.create_drip_pool(
        UncheckedDripToken::Native {
            denom: test_lab.native.clone(), 
            initial_amount: Uint128::zero() 
        },
        Uint128::new(1_000_000),
        10u64,
        &[],  
    )
    .unwrap_err()
    .downcast()
    .unwrap();

    assert_eq!(err, ContractError::ZeroTokenPool {  });

    // With cw20 pool
    let err: ContractError = test_lab.create_drip_pool(
            UncheckedDripToken::Cw20 {
                address: test_lab.cw20_address.clone(), 
                initial_amount: Uint128::zero() 
            },
            Uint128::new(1_000_000),
            10u64,    
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(err, ContractError::ZeroTokenPool {  })
    
}

#[test]
fn no_funded_contract() {
    let mut test_lab = LabBuilder::new().build();
    let owner = test_lab.owner.clone(); 
    test_lab = test_lab.init_cw20(
        vec![Cw20Coin {
            address: owner,
            amount: Uint128::new(1_000_000)
        }]
    );

    // With native pool
    let err: ContractError = test_lab.create_drip_pool(
        UncheckedDripToken::Native {
            denom: test_lab.native.to_string(), 
            initial_amount: Uint128::new(1_000_000) 
        },
        Uint128::new(1_000_000),
        10u64,
        &[],  
    )
    .unwrap_err()
    .downcast()
    .unwrap();

    assert_eq!(err, ContractError::NoFundedContract {token: test_lab.native.to_string(), amount: Uint128::new(1_000_000)});
   
    // With cw20 pool
    let cw20_addr = test_lab.cw20_address.clone();
    let err: ContractError = test_lab.create_drip_pool(
        UncheckedDripToken::Cw20 {
            address: cw20_addr, 
            initial_amount: Uint128::new(1_000_000) 
        },
        Uint128::new(1_000_000),
        10u64,    
        &[],
    )
    .unwrap_err()
    .downcast()
    .unwrap();

    assert_eq!(err, ContractError::NoFundedContract {token: test_lab.cw20_address, amount: Uint128::new(1_000_000)});

}

#[test]
fn wrong_tokens_amount() {
    let mut test_lab = LabBuilder::new().build();
    let drip_addr = test_lab.drip_address.clone();
    let native = test_lab.native.clone();
    test_lab = test_lab
        .sudo_mint_1000(drip_addr.clone(),native.clone(), 1000u128)
        .init_cw20(vec![
            Cw20Coin {
                address: drip_addr.clone(),
                amount: Uint128::new(1_000)
            }]
        );
   
    // With native pool
    let err: ContractError = test_lab.create_drip_pool(
        UncheckedDripToken::Native {
            denom: test_lab.native.to_string(), 
            initial_amount: Uint128::new(1_000_000) 
        },
        Uint128::new(1_000_000),
        10u64,
        &[],  
    )
    .unwrap_err()
    .downcast()
    .unwrap();

    assert_eq!(
        err, 
        ContractError::WrongTokensAmount { 
            tokens_amount: Uint128::new(1_000_000), 
            total_tokens: Uint128::new(10_000_000) 
        });

    // With cw20 pool
    let cw20_addr = test_lab.cw20_address.clone();
    let err: ContractError = test_lab.create_drip_pool(
        UncheckedDripToken::Cw20 {
            address: cw20_addr, 
            initial_amount: Uint128::new(1_000) 
        },
        Uint128::new(1_000_000),
        1u64,    
        &[],
    )
    .unwrap_err()
    .downcast()
    .unwrap();

    assert_eq!(
        err, 
        ContractError::WrongTokensAmount { 
            tokens_amount: Uint128::new(1_000), 
            total_tokens: Uint128::new(1_000_000) 
        });
}

#[test]
fn funded_contract() {
    let mut test_lab = LabBuilder::new().build();
    let drip_addr = test_lab.drip_address.clone();
    let native = test_lab.native.clone();
    test_lab = test_lab
        .sudo_mint_1000(drip_addr.clone(),native.clone(), 1000u128)
        .init_cw20(vec![
            Cw20Coin {
                address:drip_addr.clone(), 
                amount: Uint128::new(1_000_000)
            }]
        );

    // With native pool
    let _resp = test_lab.create_drip_pool(
        UncheckedDripToken::Native {
            denom: test_lab.native.to_string(), 
            initial_amount: Uint128::new(10_000) 
        },
        Uint128::new(1_000),
        10u64,
        &[],    
    )
    .unwrap();

    let resp = test_lab.query_drip_pools();
    assert_eq!(resp.drip_pools.len(), 1);

    let resp = test_lab.query_drip_tokens();
    assert_eq!(resp.drip_tokens.len(), 1);

    let resp = test_lab.query_drip_pool(test_lab.native.to_string());
    assert_eq!(
        resp.drip_pool, 
        Some(DripPool {
            drip_token: DripToken::Native { 
                denom: test_lab.native.to_string(), 
                initial_amount: Uint128::new(10_000) 
            },
            actual_amount: Uint128::new(10_000),
            tokens_to_withdraw: Uint128::new(0),
            tokens_per_epoch: Uint128::new(1_000),
            issued_shares: Uint128::zero(),
            epochs_number: 10u64,
            current_epoch: 0u64,
        })
    ); 
   
    // With cw20 pool
    let cw20_addr = test_lab.cw20_address.clone();
    let _resp = test_lab.create_drip_pool(
        UncheckedDripToken::Cw20 {
            address: cw20_addr.clone(), 
            initial_amount: Uint128::new(1_000_000) 
        },
        Uint128::new(100_000),
        10u64,    
        &[],
    )
    .unwrap(); 

    let resp = test_lab.query_drip_pools();
    assert_eq!(resp.drip_pools.len(), 2);

    let resp = test_lab.query_drip_tokens();
    assert_eq!(resp.drip_tokens.len(), 2);

    let resp = test_lab.query_drip_pool(cw20_addr.clone());
    assert_eq!(
        resp.drip_pool, 
        Some(DripPool {
            drip_token: DripToken::CW20 { 
                address: cw20_addr, 
                initial_amount: Uint128::new(1_000_000) 
            },
            actual_amount: Uint128::new(1_000_000),
            tokens_to_withdraw: Uint128::new(0),
            tokens_per_epoch: Uint128::new(100_000),
            issued_shares: Uint128::zero(),
            epochs_number: 10u64,
            current_epoch: 0u64,
        })
    ); 
 
}

#[test]
fn drip_pool_already_exists() {
    let mut test_lab = LabBuilder::new().build();
    let drip_addr = test_lab.drip_address.clone();
    let native = test_lab.native.clone();
    test_lab = test_lab
        .sudo_mint_1000(drip_addr.clone(),native.clone(), 1000u128)
        .init_cw20(vec![
            Cw20Coin {
                address:drip_addr.clone(), 
                amount: Uint128::new(1_000_000)
            }]
        );

    let _resp = test_lab.create_drip_pool(
        UncheckedDripToken::Native {
            denom: test_lab.native.to_string(), 
            initial_amount: Uint128::new(10_000) 
        },
        Uint128::new(1_000),
        10u64,
        &[],    
    )
    .unwrap();

    let err: ContractError = test_lab.create_drip_pool(
        UncheckedDripToken::Native {
            denom: test_lab.native.to_string(), 
            initial_amount: Uint128::new(10_000) 
        },
        Uint128::new(1_000),
        10u64,
        &[],    
    )
    .unwrap_err()
    .downcast()
    .unwrap();

    assert_eq!(err, ContractError::DripPoolAlreadyExists {  })
 
}