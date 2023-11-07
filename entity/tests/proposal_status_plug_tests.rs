use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::*;
use entity::permission::*;
use multiversx_sc::codec::multi_types::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_returns_active_for_a_newly_created_proposal() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let proposal_id = 1;

    setup.configure_plug(100, 50);

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                POLL_DEFAULT_ID,
                MultiValueManagedVec::new(),
            );
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(10);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(ProposalStatus::Active, sc.get_proposal_status_view(proposal_id));
        })
        .assert_ok();
}

#[test]
fn it_returns_defeated_when_quorum_not_met() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voter_one = setup.blockchain.create_user_account(&rust_biguint!(1));
    let voter_two = setup.blockchain.create_user_account(&rust_biguint!(1));
    let proposal_id = 1;

    setup.configure_plug(500, 50);

    // propose with 100 votes
    setup
        .blockchain
        .execute_tx(&voter_one, &setup.contract, &rust_biguint!(0), |sc| {
            sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                POLL_DEFAULT_ID,
                MultiValueManagedVec::new(),
            );
        })
        .assert_ok();

    // vote with 100
    setup
        .blockchain
        .execute_tx(&voter_two, &setup.contract, &rust_biguint!(0), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    // defeat because quorum was configured to 500 votes
    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let proposal = sc.proposals(proposal_id).get();

            assert_eq!(managed_biguint!(200), proposal.votes_for);
            assert_eq!(ProposalStatus::Defeated, sc.get_proposal_status_view(proposal_id));
        })
        .assert_ok();
}

#[test]
fn it_returns_defeated_when_quorum_met_but_votes_against_is_more_than_for() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voter_one = setup.blockchain.create_user_account(&rust_biguint!(1));
    let voter_two = setup.blockchain.create_user_account(&rust_biguint!(1));
    let voter_three = setup.blockchain.create_user_account(&rust_biguint!(1));
    let proposal_id = 1;

    setup.configure_plug(100, 50);

    // propose FOR with 100 votes
    setup
        .blockchain
        .execute_tx(&voter_one, &setup.contract, &rust_biguint!(0), |sc| {
            sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                POLL_DEFAULT_ID,
                MultiValueManagedVec::new(),
            );
        })
        .assert_ok();

    // vote AGAINST with 100
    setup
        .blockchain
        .execute_tx(&voter_two, &setup.contract, &rust_biguint!(0), |sc| {
            sc.vote_against_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    // vote AGAINST with 100
    setup
        .blockchain
        .execute_tx(&voter_three, &setup.contract, &rust_biguint!(0), |sc| {
            sc.vote_against_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let proposal = sc.proposals(proposal_id).get();

            assert_eq!(managed_biguint!(100), proposal.votes_for);
            assert_eq!(managed_biguint!(200), proposal.votes_against);
            assert_eq!(ProposalStatus::Defeated, sc.get_proposal_status_view(proposal_id));
        })
        .assert_ok();
}

#[test]
fn it_returns_succeeded_when_for_votes_quorum_met_and_more_for_than_against_votes() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voter_one = setup.blockchain.create_user_account(&rust_biguint!(1));
    let voter_two = setup.blockchain.create_user_account(&rust_biguint!(1));
    let voter_three = setup.blockchain.create_user_account(&rust_biguint!(1));
    let proposal_id = 1;

    setup.configure_plug(10, 50);

    // propose FOR with 100 votes
    setup
        .blockchain
        .execute_tx(&voter_one, &setup.contract, &rust_biguint!(0), |sc| {
            sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                POLL_DEFAULT_ID,
                MultiValueManagedVec::new(),
            );
        })
        .assert_ok();

    // vote FOR with 100
    setup
        .blockchain
        .execute_tx(&voter_two, &setup.contract, &rust_biguint!(0), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    // vote AGAINST with 100
    setup
        .blockchain
        .execute_tx(&voter_three, &setup.contract, &rust_biguint!(0), |sc| {
            sc.vote_against_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let proposal = sc.proposals(proposal_id).get();

            assert_eq!(managed_biguint!(200), proposal.votes_for);
            assert_eq!(managed_biguint!(100), proposal.votes_against);
            assert_eq!(ProposalStatus::Succeeded, sc.get_proposal_status_view(proposal_id));
        })
        .assert_ok();
}

#[test]
fn it_returns_executed_for_an_executed_proposal() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.blockchain.create_user_account(&rust_biguint!(1));
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let proposal_id = 1;

    setup.configure_plug(10, 50);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
            sc.create_permission(
                managed_buffer!(b"perm"),
                managed_biguint!(0),
                managed_address!(&action_receiver),
                managed_buffer!(b"myendpoint"),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_policy(
                managed_buffer!(ROLE_BUILTIN_LEADER),
                managed_buffer!(b"perm"),
                PolicyMethod::Quorum,
                BigUint::from(1u64),
                10,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"perm")]);

            sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            sc.execute_endpoint(proposal_id, MultiValueManagedVec::from(actions));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(ProposalStatus::Executed, sc.get_proposal_status_view(proposal_id));
        })
        .assert_ok();
}
