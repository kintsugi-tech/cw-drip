use cosmwasm_std::Addr;

use crate::{tests::environment::{PAR1, PAR2, PAR3, LabBuilder}, ContractError};


#[test]
fn participant() {
    let mut test_lab = LabBuilder::new().build();

    let participant = Addr::unchecked(PAR1);
    let _resp = test_lab.add_participant(participant.clone()).unwrap();

    let resp = test_lab.query_participants();
    assert_eq!(resp.participants.len(), 1);
    assert_eq!(resp.participants[0], participant);

    let err: ContractError = test_lab.add_participant(participant.clone())
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ContractError::AlreadyParticipant {})
}

#[test]
fn remove_participant() {
    let mut test_lab = LabBuilder::new().build();

    let participant = Addr::unchecked(PAR1);
    let _resp = test_lab.add_participant(participant.clone()).unwrap();
    let _resp = test_lab.remove_participant(participant).unwrap();

    let resp = test_lab.query_participants();
    assert_eq!(resp.participants.len(), 0);

    // Remove non existing participant will not cause errors
    let _resp = test_lab.remove_participant(Addr::unchecked("blablabla"))
        .unwrap();

    let resp = test_lab.query_participants();
    assert_eq!(resp.participants.len(), 0); 
}

#[test]
fn participants() {
    let mut test_lab = LabBuilder::new().build();

    let participant1 = Addr::unchecked(PAR1);
    let _resp = test_lab.add_participant(participant1.clone()).unwrap();

    let participant2 = Addr::unchecked(PAR2);
    let _resp = test_lab.add_participant(participant2.clone()).unwrap();
    
    let resp = test_lab.query_participants();
    assert_eq!(resp.participants.len(), 2);
    assert_eq!(resp.participants, vec![participant1.clone(), participant2.clone()]);

    let _resp = test_lab.remove_participant(participant1);

    let participant3 = Addr::unchecked(PAR3);
    let _resp = test_lab.add_participant(participant3.clone()).unwrap();
    let resp = test_lab.query_participants();
    assert_eq!(resp.participants.len(), 2);
    assert_eq!(resp.participants, vec![participant2.clone(), participant3.clone()]);

} 