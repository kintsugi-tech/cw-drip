use cosmwasm_std::{Addr, Coin, Uint128};
use cw20::Cw20Coin;

use crate::{
    msg::UncheckedDripToken,
    state::{DripPool, DripToken},
    ContractError,
};

use super::lab::{LabBuilder, EPOCH, PAR1, PAR2, PAR3};

#[test]
pub fn zero_active_pool() {
    let mut test_lab = LabBuilder::new().build();
    let err: ContractError = test_lab
        .distribute_shares()
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(err, ContractError::ZeroActiveDripPool {})
}

#[test]
pub fn no_distribution_time() {
    let mut test_lab = LabBuilder::new().build();
    let drip_addr = test_lab.drip_address.clone();
    let native = test_lab.native.clone();
    test_lab = test_lab
        .sudo_mint_1000(drip_addr.clone(), native.clone(), 100u128)
        .sudo_mint_1000(PAR1.to_string(), native.clone(), 1_000u128)
        .init_cw20(vec![Cw20Coin {
            address: drip_addr.clone(),
            amount: Uint128::new(1_000_000),
        }]);

    // With native pool
    let _resp = test_lab
        .create_drip_pool(
            UncheckedDripToken::Native {
                denom: test_lab.native.to_string(),
                initial_amount: Uint128::new(10_000),
            },
            Uint128::new(1_000),
            10u64,
            &[],
        )
        .unwrap();

    let err: ContractError = test_lab
        .distribute_shares()
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(err, ContractError::NoDistributionTime {})
}

#[test]
pub fn no_min_staking() {
    let mut test_lab = LabBuilder::new().build();
    let drip_addr = test_lab.drip_address.clone();
    let native = test_lab.native.clone();
    test_lab = test_lab
        .sudo_mint_1000(drip_addr.clone(), native.clone(), 100u128)
        .sudo_mint_1000(PAR1.to_string(), native.clone(), 1_000u128)
        .init_cw20(vec![Cw20Coin {
            address: drip_addr.clone(),
            amount: Uint128::new(1_000_000),
        }]);

    _ = test_lab.create_delegation(
        Addr::unchecked(PAR1),
        "validator1".to_string(),
        Coin {
            denom: native.clone(),
            amount: Uint128::new(1_000),
        },
    );

    // With native pool
    let _resp = test_lab
        .create_drip_pool(
            UncheckedDripToken::Native {
                denom: test_lab.native.to_string(),
                initial_amount: Uint128::new(10_000),
            },
            Uint128::new(1_000),
            10u64,
            &[],
        )
        .unwrap();

    let resp = test_lab.query_drip_pool(test_lab.native.to_string());
    assert_eq!(
        resp.drip_pool,
        Some(DripPool {
            drip_token: DripToken::Native {
                denom: test_lab.native.to_string(),
                amount: Uint128::new(10_000)
            },
            initial_amount: Uint128::new(10_000),
            withdrawable_tokens: Uint128::new(0),
            tokens_per_epoch: Uint128::new(1_000),
            issued_shares: Uint128::zero(),
            epochs_number: 10u64,
            epoch: 0u64,
        })
    );

    test_lab.advance_blocks(EPOCH);

    let _resp = test_lab.distribute_shares().unwrap();

    let resp = test_lab.query_drip_pool(native.clone());
    assert_eq!(
        resp.drip_pool.map(|pool| pool.issued_shares).unwrap(),
        Uint128::zero()
    );
}

#[test]
pub fn distribute_single() {
    let mut test_lab = LabBuilder::new().build();
    let drip_addr = test_lab.drip_address.clone();
    let native = test_lab.native.clone();
    test_lab = test_lab
        .sudo_mint_1000(drip_addr.clone(), native.clone(), 100u128)
        .sudo_mint_1000(PAR1.to_string(), native.clone(), 1_000u128)
        .init_cw20(vec![Cw20Coin {
            address: drip_addr.clone(),
            amount: Uint128::new(1_000_000),
        }]);

    let shares = Uint128::new(1_000_000);

    _ = test_lab.create_delegation(
        Addr::unchecked(PAR1),
        "validator1".to_string(),
        Coin {
            denom: native.clone(),
            amount: shares,
        },
    );

    let participant1 = Addr::unchecked(PAR1);
    let _resp = test_lab.add_participant(participant1.clone()).unwrap();

    // With native pool
    let _resp = test_lab
        .create_drip_pool(
            UncheckedDripToken::Native {
                denom: test_lab.native.to_string(),
                initial_amount: Uint128::new(10_000),
            },
            Uint128::new(1_000),
            10u64,
            &[],
        )
        .unwrap();

    test_lab.advance_blocks(EPOCH);

    let _resp = test_lab.distribute_shares().unwrap();

    let resp = test_lab.query_drip_pool(native.clone());
    if let Some(pool) = resp.drip_pool {
        assert_eq!(pool.drip_token.get_available_amount(), Uint128::new(9_000));
        assert_eq!(pool.issued_shares, Uint128::new(1_000_000));
        assert_eq!(pool.withdrawable_tokens, Uint128::new(1_000));
    }

    let resp = test_lab.query_participant_shares(PAR1.to_string());
    assert_eq!(resp.shares.len(), 1)
}

#[test]
pub fn distribute_multiple() {
    let mut test_lab = LabBuilder::new().build();
    let drip_addr = test_lab.drip_address.clone();
    let native = test_lab.native.clone();
    test_lab = test_lab
        .sudo_mint_1000(drip_addr.clone(), native.clone(), 100u128)
        .sudo_mint_1000(PAR1.to_string(), native.clone(), 1_000u128)
        .sudo_mint_1000(PAR2.to_string(), native.clone(), 2_000u128)
        .sudo_mint_1000(PAR3.to_string(), native.clone(), 3_000u128)
        .init_cw20(vec![Cw20Coin {
            address: drip_addr.clone(),
            amount: Uint128::new(1_000_000),
        }]);

    _ = test_lab.create_delegation(
        Addr::unchecked(PAR1),
        "validator1".to_string(),
        Coin {
            denom: native.clone(),
            amount: Uint128::new(1_000_000),
        },
    );

    _ = test_lab.create_delegation(
        Addr::unchecked(PAR2),
        "validator1".to_string(),
        Coin {
            denom: native.clone(),
            amount: Uint128::new(2_000_000),
        },
    );

    _ = test_lab.create_delegation(
        Addr::unchecked(PAR3),
        "validator1".to_string(),
        Coin {
            denom: native.clone(),
            amount: Uint128::new(3_000_000),
        },
    );

    let participant1 = Addr::unchecked(PAR1);
    let _resp = test_lab.add_participant(participant1.clone()).unwrap();

    let participant2 = Addr::unchecked(PAR2);
    let _resp = test_lab.add_participant(participant2.clone()).unwrap();

    let participant3 = Addr::unchecked(PAR3);
    let _resp = test_lab.add_participant(participant3.clone()).unwrap();

    // With native pool
    let _resp = test_lab
        .create_drip_pool(
            UncheckedDripToken::Native {
                denom: test_lab.native.to_string(),
                initial_amount: Uint128::new(10_000),
            },
            Uint128::new(1_000),
            10u64,
            &[],
        )
        .unwrap();

    // First distribution
    test_lab.advance_blocks(EPOCH);

    let _resp = test_lab.distribute_shares().unwrap();

    let resp = test_lab.query_drip_pool(native.clone());
    if let Some(pool) = resp.drip_pool {
        assert_eq!(pool.drip_token.get_available_amount(), Uint128::new(9_000));
        assert_eq!(pool.issued_shares, Uint128::new(6_000_000));
        assert_eq!(pool.withdrawable_tokens, Uint128::new(1_000));
    }

    let resp = test_lab.query_participant_shares(PAR1.to_string());
    assert_eq!(resp.shares.len(), 1);

    let resp = test_lab.query_participant_shares(PAR2.to_string());
    assert_eq!(resp.shares.len(), 1);

    let resp = test_lab.query_participant_shares(PAR3.to_string());
    assert_eq!(resp.shares.len(), 1);

    // Second distribution
    test_lab.advance_blocks(EPOCH);

    let _resp = test_lab.distribute_shares().unwrap();

    let resp = test_lab.query_drip_pool(native.clone());
    if let Some(pool) = resp.drip_pool {
        assert_eq!(pool.drip_token.get_available_amount(), Uint128::new(8_000));
        assert_eq!(pool.issued_shares, Uint128::new(12_000_000));
        assert_eq!(pool.withdrawable_tokens, Uint128::new(2_000));
    }

    let resp = test_lab.query_participant_shares(PAR1.to_string());
    assert_eq!(resp.shares.len(), 1);

    let resp = test_lab.query_participant_shares(PAR2.to_string());
    assert_eq!(resp.shares.len(), 1);

    // Distribute all drip tokens
    let mut i = 0;
    while i < 8 {
        test_lab.advance_blocks(EPOCH);

        let _resp = test_lab.distribute_shares().unwrap();

        i += 1;
    }
    let resp = test_lab.query_drip_pool(native.clone());
    if let Some(pool) = resp.drip_pool {
        assert_eq!(pool.drip_token.get_available_amount(), Uint128::new(0));
        assert_eq!(pool.issued_shares, Uint128::new(10 * 6_000_000));
        assert_eq!(pool.withdrawable_tokens, Uint128::new(10_000));
    }

    let resp = test_lab.query_drip_pool(native.clone());
    if let Some(pool) = resp.drip_pool {
        assert_eq!(pool.drip_token.get_available_amount(), Uint128::new(0));
        assert_eq!(pool.issued_shares, Uint128::new(10 * 6_000_000));
        assert_eq!(pool.withdrawable_tokens, Uint128::new(10_000));
    }

    // Another distribution round will throw an error.
    test_lab.advance_blocks(EPOCH);
    let err: ContractError = test_lab
        .distribute_shares()
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(err, ContractError::ZeroActiveDripPool {});
}

#[test]
pub fn multiple_drip_pools() {
    let mut test_lab = LabBuilder::new().build();
    let drip_addr = test_lab.drip_address.clone();
    let native = test_lab.native.clone();
    test_lab = test_lab
        .sudo_mint_1000(drip_addr.clone(), native.clone(), 100u128)
        .sudo_mint_1000(PAR1.to_string(), native.clone(), 1_000u128)
        .init_cw20(vec![Cw20Coin {
            address: drip_addr.clone(),
            amount: Uint128::new(1_000_000),
        }]);

    let shares = Uint128::new(1_000_000);

    _ = test_lab.create_delegation(
        Addr::unchecked(PAR1),
        "validator1".to_string(),
        Coin {
            denom: native.clone(),
            amount: shares,
        },
    );

    let participant1 = Addr::unchecked(PAR1);
    let _resp = test_lab.add_participant(participant1.clone()).unwrap();

    // With native pool
    let _resp = test_lab
        .create_drip_pool(
            UncheckedDripToken::Native {
                denom: test_lab.native.to_string(),
                initial_amount: Uint128::new(10_000),
            },
            Uint128::new(1_000),
            10u64,
            &[],
        )
        .unwrap();

    test_lab.advance_blocks(EPOCH);

    let _resp = test_lab.distribute_shares().unwrap();

    let resp = test_lab.query_drip_pool(native.clone());
    assert_eq!(
        resp.drip_pool.map(|pool| pool.issued_shares).unwrap(),
        shares
    );

    let resp = test_lab.query_participant_shares(PAR1.to_string());
    assert_eq!(resp.shares.len(), 1);

    // Introduce second drip pool
    let _resp = test_lab
        .create_drip_pool(
            UncheckedDripToken::Cw20 {
                address: test_lab.cw20_address.clone(),
                initial_amount: Uint128::new(50_000),
            },
            Uint128::new(25_000),
            2u64,
            &[],
        )
        .unwrap();

    test_lab.advance_blocks(EPOCH);

    let _resp = test_lab.distribute_shares().unwrap();

    let resp = test_lab.query_participant_shares(PAR1.to_string());
    assert_eq!(resp.shares.len(), 2);
    assert_eq!(
        resp.shares,
        vec![
            (test_lab.cw20_address.clone(), Uint128::new(1_000_000)),
            (test_lab.native.clone(), Uint128::new(2_000_000))
        ]
    );
}
