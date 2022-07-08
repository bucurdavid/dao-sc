use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::*;
use entity::permission::*;
use setup::*;

mod setup;

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
fn it_succeeds_early_if_has_all_required_signatures_for_proposal_with_actions() {
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
    }).assert_ok();

    setup.blockchain.execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
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

    setup.blockchain.execute_tx(&signer_one, &setup.contract, &rust_biguint!(0), |sc| {
        sc.sign_endpoint(proposal_id);
    })
    .assert_ok();

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Succeeded, sc.get_proposal_status_view(proposal_id));
    })
    .assert_ok();
}

#[test]
fn it_returns_executed_for_an_executed_proposal_with_signer_quorum() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.blockchain.create_user_account(&rust_biguint!(1));
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let signer_one = setup.blockchain.create_user_account(&rust_biguint!(1));
    let signer_inactive = setup.blockchain.create_user_account(&rust_biguint!(1));
    let mut proposal_id = 0;

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.create_role(managed_buffer!(b"builder"));
        sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
        sc.assign_role(managed_address!(&signer_one), managed_buffer!(b"builder"));
        sc.assign_role(managed_address!(&signer_inactive), managed_buffer!(b"builder"));
    }).assert_ok();

    setup.blockchain.execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
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

    setup.blockchain.execute_tx(&signer_one, &setup.contract, &rust_biguint!(0), |sc| {
        sc.sign_endpoint(proposal_id);
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
