use cosmwasm_std::{Addr, Coin, Uint128};

use crate::{
    tests::lab::{LabBuilder, MIN_STAKING, PAR1, PAR2, PAR3},
    ContractError,
};

#[test]
fn participant() {
    let mut test_lab = LabBuilder::new().build();
    let native = test_lab.native.clone();
    let participant = Addr::unchecked(PAR1);

    test_lab = test_lab.sudo_mint_1000(PAR1.to_string(), native.clone(), 1_000u128);

    let err: ContractError = test_lab
        .add_participant(participant.clone())
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(
        err,
        ContractError::MinimumDelegationNotSatisfied {
            min_staked: MIN_STAKING
        }
    );

    _ = test_lab.create_delegation(
        Addr::unchecked(PAR1),
        "validator1".to_string(),
        Coin {
            denom: native,
            amount: Uint128::new(1_000_000),
        },
    );

    let _resp = test_lab.add_participant(participant.clone()).unwrap();

    let resp = test_lab.query_participants();
    assert_eq!(resp.participants.len(), 1);
    assert_eq!(resp.participants[0], participant);

    let resp = test_lab.query_participants();
    assert_eq!(resp.participants.len(), 1);
    assert_eq!(resp.participants[0], participant);

    let err: ContractError = test_lab
        .add_participant(participant)
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ContractError::AlreadyParticipant {});
}

#[test]
fn remove_participant() {
    let mut test_lab = LabBuilder::new().build();

    let native = test_lab.native.clone();
    let participant = Addr::unchecked(PAR1);

    test_lab = test_lab.sudo_mint_1000(PAR1.to_string(), native.clone(), 1_000u128);

    _ = test_lab.create_delegation(
        Addr::unchecked(PAR1),
        "validator1".to_string(),
        Coin {
            denom: native,
            amount: Uint128::new(1_000_000),
        },
    );

    let _resp = test_lab.add_participant(participant.clone()).unwrap();
    let _resp = test_lab.remove_participant(participant).unwrap();

    let resp = test_lab.query_participants();
    assert_eq!(resp.participants.len(), 0);

    // Remove non existing participant will not cause errors
    let _resp = test_lab
        .remove_participant(Addr::unchecked("blablabla"))
        .unwrap();

    let resp = test_lab.query_participants();
    assert_eq!(resp.participants.len(), 0);
}

#[test]
fn participants() {
    let mut test_lab = LabBuilder::new().build();
    let native = test_lab.native.clone();

    test_lab = test_lab
        .sudo_mint_1000(PAR1.to_string(), native.clone(), 1_000u128)
        .sudo_mint_1000(PAR2.to_string(), native.clone(), 1_000u128)
        .sudo_mint_1000(PAR3.to_string(), native.clone(), 1_000u128);

    _ = test_lab.create_delegation(
        Addr::unchecked(PAR1),
        "validator1".to_string(),
        Coin {
            denom: native.clone(),
            amount: Uint128::new(1_000_000),
        },
    );

    let participant1 = Addr::unchecked(PAR1);
    let _resp = test_lab.add_participant(participant1.clone()).unwrap();

    _ = test_lab.create_delegation(
        Addr::unchecked(PAR2),
        "validator1".to_string(),
        Coin {
            denom: native.clone(),
            amount: Uint128::new(1_000_000),
        },
    );

    let participant2 = Addr::unchecked(PAR2);
    let _resp = test_lab.add_participant(participant2.clone()).unwrap();

    let resp = test_lab.query_participants();
    assert_eq!(resp.participants.len(), 2);
    assert_eq!(
        resp.participants,
        vec![participant1.clone(), participant2.clone()]
    );

    let _resp = test_lab.remove_participant(participant1);

    let participant3 = Addr::unchecked(PAR3);

    _ = test_lab.create_delegation(
        Addr::unchecked(PAR3),
        "validator1".to_string(),
        Coin {
            denom: native,
            amount: Uint128::new(1_000_000),
        },
    );

    let _resp = test_lab.add_participant(participant3.clone()).unwrap();
    let resp = test_lab.query_participants();
    assert_eq!(resp.participants.len(), 2);
    assert_eq!(resp.participants, vec![participant2, participant3]);
}
