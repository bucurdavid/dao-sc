use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::*;

mod setup;

#[test]
fn it_returns_active_for_a_newly_created_proposal() {
    let mut setup = setup::setup_entity(entity::contract_obj);

    setup.blockchain.set_block_timestamp(0);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let starts_at = 0u64;
            let voting_period_minutes = sc.voting_period_in_minutes().get() as u64;
            let ends_at = starts_at + voting_period_minutes * 60;

            let dummy_proposal = Proposal::<DebugApi> {
                actions_hash: managed_buffer!(b""),
                starts_at,
                ends_at,
                content_hash: managed_buffer!(b""),
                id: 0,
                votes_against: managed_biguint!(0),
                votes_for: managed_biguint!(0),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(0).set(dummy_proposal);
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let status = sc.get_proposal_status_view(0);
            assert_eq!(ProposalStatus::Active, status);
        })
        .assert_ok();
}

#[test]
fn it_returns_defeated_if_for_votes_quorum_not_met() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let starts_at = 0u64;
            let ends_at = starts_at + voting_period_seconds;

            let dummy_proposal = Proposal::<DebugApi> {
                actions_hash: managed_buffer!(b""),
                starts_at,
                ends_at,
                content_hash: managed_buffer!(b""),
                id: 0,
                votes_against: managed_biguint!(0),
                votes_for: sc.quorum().get() - BigUint::from(1u64),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(0).set(dummy_proposal);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let status = sc.get_proposal_status_view(0);
            assert_eq!(ProposalStatus::Defeated, status);
        })
        .assert_ok();
}

#[test]
fn it_returns_defeated_if_votes_against_is_more_than_for() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let starts_at = 0u64;
            let ends_at = starts_at + voting_period_seconds;

            let dummy_proposal = Proposal::<DebugApi> {
                actions_hash: managed_buffer!(b""),
                starts_at,
                ends_at,
                content_hash: managed_buffer!(b""),
                id: 0,
                votes_against: sc.quorum().get() + BigUint::from(2u64),
                votes_for: sc.quorum().get() + BigUint::from(1u64),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(0).set(dummy_proposal);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let status = sc.get_proposal_status_view(0);
            assert_eq!(ProposalStatus::Defeated, status);
        })
        .assert_ok();
}

#[test]
fn it_returns_succeeded_if_for_votes_quorum_met_and_more_for_than_against_votes() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let starts_at = 0u64;
            let ends_at = starts_at + voting_period_seconds;

            let dummy_proposal = Proposal::<DebugApi> {
                actions_hash: managed_buffer!(b""),
                starts_at,
                ends_at,
                content_hash: managed_buffer!(b""),
                id: 0,
                votes_against: sc.quorum().get() + BigUint::from(0u64),
                votes_for: sc.quorum().get() + BigUint::from(5u64),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(0).set(dummy_proposal);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let status = sc.get_proposal_status_view(0);
            assert_eq!(ProposalStatus::Succeeded, status);
        })
        .assert_ok();
}

#[test]
fn it_returns_executed_for_an_executed_proposal() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let starts_at = 0u64;
            let ends_at = starts_at + voting_period_seconds;

            let dummy_proposal = Proposal::<DebugApi> {
                actions_hash: managed_buffer!(b""),
                starts_at,
                ends_at,
                content_hash: managed_buffer!(b""),
                id: 0,
                votes_against: sc.quorum().get() + BigUint::from(0u64),
                votes_for: sc.quorum().get() + BigUint::from(5u64),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(0).set(dummy_proposal);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.execute_endpoint(0, MultiValueManagedVec::new());
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let status = sc.get_proposal_status_view(0);
            assert_eq!(ProposalStatus::Executed, status);
        })
        .assert_ok();
}
