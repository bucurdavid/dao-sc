use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::*;
use entity::permission::*;
use setup::*;

mod setup;

#[test]
fn it_returns_active_for_a_newly_created_proposal() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let mut proposal_id = 0;

    setup.configure_gov_token();

    setup.blockchain.set_block_timestamp(0);

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), managed_buffer!(b""), MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Active, sc.get_proposal_status_view(proposal_id));
    })
    .assert_ok();
}

#[test]
fn it_returns_defeated_if_for_votes_quorum_not_met() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let mut proposal_id = 0;

    setup.configure_gov_token();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM - 10), |sc| {
        proposal_id= sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), managed_buffer!(b""), MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(9), |sc| {
        sc.vote_for_endpoint(proposal_id);
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Defeated, sc.get_proposal_status_view(proposal_id));
    })
    .assert_ok();
}

#[test]
fn it_returns_defeated_if_quorum_met_but_votes_against_is_more_than_for() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let mut proposal_id = 0;

    setup.configure_gov_token();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(10), |sc| {
        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), managed_buffer!(b""), MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
        sc.vote_for_endpoint(proposal_id);
    })
    .assert_ok();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM * 2), |sc| {
        sc.vote_against_endpoint(proposal_id);
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Defeated, sc.get_proposal_status_view(proposal_id));
    })
    .assert_ok();
}

#[test]
fn it_applies_the_default_quorum_to_proposals_with_actions_that_do_not_require_any_permissions() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.configure_gov_token();

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.assign_role(managed_address!(&proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(10), |sc| {
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

        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), actions_hash, MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Defeated, sc.get_proposal_status_view(proposal_id));
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(1); // go back in time

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
        sc.vote_for_endpoint(proposal_id);
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Succeeded, sc.get_proposal_status_view(proposal_id));
    })
    .assert_ok();
}

#[test]
fn it_requires_signer_majority_if_proposer_has_role_and_with_actions_that_do_not_require_any_permissions() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let signer_one = setup.blockchain.create_user_account(&rust_biguint!(1));
    let signer_inactive = setup.blockchain.create_user_account(&rust_biguint!(1));
    let mut proposal_id = 0;

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.create_role(managed_buffer!(b"builder"));
        sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
        sc.assign_role(managed_address!(&signer_one), managed_buffer!(b"builder"));
        sc.assign_role(managed_address!(&signer_inactive), managed_buffer!(b"builder"));

        sc.create_permission(managed_buffer!(b"perm"), managed_address!(&action_receiver), managed_buffer!(b"any"), ManagedVec::new());
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(1), |sc| {
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

        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), actions_hash, actions_permissions);
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Defeated, sc.get_proposal_status_view(proposal_id));
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(1); // go back in time

    setup.blockchain.execute_tx(&signer_one, &setup.contract, &rust_biguint!(0), |sc| {
        sc.sign_endpoint(proposal_id);
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Succeeded, sc.get_proposal_status_view(proposal_id));
    })
    .assert_ok();
}

#[test]
fn it_returns_succeeded_if_for_votes_quorum_met_and_more_for_than_against_votes() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let mut proposal_id = 0;

    setup.configure_gov_token();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), managed_buffer!(b""), MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(20), |sc| {
        sc.vote_for_endpoint(proposal_id);
    })
    .assert_ok();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(10), |sc| {
        sc.vote_against_endpoint(proposal_id);
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Succeeded, sc.get_proposal_status_view(proposal_id));
    })
    .assert_ok();
}

#[test]
fn it_returns_executed_for_an_executed_proposal() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.blockchain.create_user_account(&rust_biguint!(1));
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.assign_role(managed_address!(&proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
        sc.create_permission(managed_buffer!(b"perm"), managed_address!(&action_receiver), managed_buffer!(b"myendpoint"), ManagedVec::new());
        sc.create_policy(managed_buffer!(ROLE_BUILTIN_LEADER), managed_buffer!(b"perm"), PolicyMethod::Quorum, BigUint::from(1u64), 10);
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
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

        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), actions_hash, actions_permissions);
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
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

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Executed, sc.get_proposal_status_view(1));
    })
    .assert_ok();
}
