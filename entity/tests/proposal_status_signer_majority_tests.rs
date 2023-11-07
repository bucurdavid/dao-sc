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
fn it_requires_signer_majority_when_proposer_has_role_and_with_actions_that_do_not_require_any_permissions() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.owner_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let signer_one = setup.blockchain.create_user_account(&rust_biguint!(1));
    let signer_inactive = setup.blockchain.create_user_account(&rust_biguint!(1));
    let mut proposal_id = 0;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&signer_one), managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&signer_inactive), managed_buffer!(b"builder"));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
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

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                MultiValueManagedVec::new(),
            );
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(ProposalStatus::Defeated, sc.get_proposal_status_view(proposal_id));
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(1); // go back in time

    setup
        .blockchain
        .execute_tx(&signer_one, &setup.contract, &rust_biguint!(0), |sc| {
            sc.sign_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(2, sc.get_signer_majority_for_role(&managed_buffer!(b"builder")));
            assert_eq!(ProposalStatus::Succeeded, sc.get_proposal_status_view(proposal_id));
        })
        .assert_ok();
}

#[test]
fn it_fails_when_signer_majority_not_met_when_proposer_has_role_and_with_actions_that_do_not_require_any_permissions() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.owner_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let signer_inactive_one = setup.blockchain.create_user_account(&rust_biguint!(1));
    let signer_inactive_two = setup.blockchain.create_user_account(&rust_biguint!(1));
    let mut proposal_id = 0;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&signer_inactive_one), managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&signer_inactive_two), managed_buffer!(b"builder"));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
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

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                MultiValueManagedVec::new(),
            );
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(ProposalStatus::Defeated, sc.get_proposal_status_view(proposal_id));
        })
        .assert_ok();
}

#[test]
fn it_requires_signer_majority_for_multiple_roles() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.owner_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let signer = setup.blockchain.create_user_account(&rust_biguint!(1));
    let signer_dev_one = setup.blockchain.create_user_account(&rust_biguint!(1));
    let signer_dev_two_inactive = setup.blockchain.create_user_account(&rust_biguint!(1));
    let signer_dev_three_inactive = setup.blockchain.create_user_account(&rust_biguint!(1));
    let mut proposal_id = 0;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.create_role(managed_buffer!(b"dev"));

            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&signer), managed_buffer!(b"builder"));

            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"dev"));
            sc.assign_role(managed_address!(&signer), managed_buffer!(b"dev"));
            sc.assign_role(managed_address!(&signer_dev_one), managed_buffer!(b"dev"));
            sc.assign_role(managed_address!(&signer_dev_two_inactive), managed_buffer!(b"dev"));
            sc.assign_role(managed_address!(&signer_dev_three_inactive), managed_buffer!(b"dev"));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
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

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                MultiValueManagedVec::new(),
            );
        })
        .assert_ok();

    // builder: 2 / 2
    // dev: 2 / 5
    setup
        .blockchain
        .execute_tx(&signer, &setup.contract, &rust_biguint!(0), |sc| {
            sc.sign_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(ProposalStatus::Defeated, sc.get_proposal_status_view(proposal_id));
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(1); // go back in time

    // builder: 2 / 2
    // dev: 3 / 5 -> majority -> succeed instantly
    setup
        .blockchain
        .execute_tx(&signer_dev_one, &setup.contract, &rust_biguint!(0), |sc| {
            sc.sign_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(2, sc.get_signer_majority_for_role(&managed_buffer!(b"builder")));
            assert_eq!(ProposalStatus::Succeeded, sc.get_proposal_status_view(proposal_id));
        })
        .assert_ok();
}

#[test]
fn it_succeeds_early_when_has_all_required_signatures_for_proposal_with_actions() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.owner_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let signer_one = setup.blockchain.create_user_account(&rust_biguint!(1));
    let signer_inactive = setup.blockchain.create_user_account(&rust_biguint!(1));
    let mut proposal_id = 0;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&signer_one), managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&signer_inactive), managed_buffer!(b"builder"));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
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

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                MultiValueManagedVec::new(),
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&signer_one, &setup.contract, &rust_biguint!(0), |sc| {
            sc.sign_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(ProposalStatus::Succeeded, sc.get_proposal_status_view(proposal_id));
        })
        .assert_ok();
}

#[test]
fn it_returns_executed_for_an_executed_proposal_with_signer_quorum() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.owner_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let signer_one = setup.blockchain.create_user_account(&rust_biguint!(1));
    let signer_inactive = setup.blockchain.create_user_account(&rust_biguint!(1));
    let mut proposal_id = 0;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
            sc.assign_role(managed_address!(&signer_one), managed_buffer!(ROLE_BUILTIN_LEADER));
            sc.assign_role(managed_address!(&signer_inactive), managed_buffer!(ROLE_BUILTIN_LEADER));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
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

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                MultiValueManagedVec::new(),
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&signer_one, &setup.contract, &rust_biguint!(0), |sc| {
            sc.sign_endpoint(proposal_id, OptionalValue::None);
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
