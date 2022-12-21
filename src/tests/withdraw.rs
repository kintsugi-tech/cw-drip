use cosmwasm_std::{Addr, Coin, Uint128};
use cw20::Cw20Coin;

use crate::{msg::UncheckedDripToken, ContractError};

use super::lab::{LabBuilder, EPOCH, PAR1};

#[test]
pub fn withdraw_single() {
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

    let resp = test_lab.query_balance(PAR1.into());
    assert_eq!(resp, Uint128::zero());

    let _resp = test_lab.withdraw_tokens(Addr::unchecked(PAR1)).unwrap();

    let resp = test_lab.query_balance(PAR1.into());
    assert_eq!(resp, Uint128::new(1_000));

    let err: ContractError = test_lab
        .withdraw_tokens(Addr::unchecked(PAR1))
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ContractError::NoTokensToWithdraw {})
}

#[test]
pub fn withdraw_multiple() {
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

    let resp = test_lab.query_cw20_balance(PAR1.into());
    assert_eq!(resp, Uint128::zero());

    let _resp = test_lab.withdraw_tokens(Addr::unchecked(PAR1)).unwrap();

    let resp = test_lab.query_balance(PAR1.into());
    assert_eq!(resp, Uint128::new(1_000));
    let resp = test_lab.query_cw20_balance(PAR1.into());
    assert_eq!(resp, Uint128::new(25_000));
}
