use std::vec;

use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::*;
use entity::permission::*;
use setup::*;

mod setup;

#[test]
fn it_matches_a_permission_based_on_destination_only() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let executor_address = setup.blockchain.create_user_account(&rust_biguint!(5));
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
            sc.create_permission(
                managed_buffer!(b"addressOnlyPerm"),
                managed_biguint!(0),
                managed_address!(&action_receiver),
                managed_buffer!(b""),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_policy(
                managed_buffer!(b"builder"),
                managed_buffer!(b"addressOnlyPerm"),
                PolicyMethod::All,
                BigUint::from(0u64),
                10,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
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
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"addressOnlyPerm")]);

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b""),
                managed_buffer!(b""),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&executor_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            let proposal = sc.proposals(proposal_id).get();

            let actual = sc.get_actual_permissions(&proposal, &ManagedVec::from(actions));

            assert_eq!(1, actual.len());
        })
        .assert_ok();
}

#[test]
fn it_matches_a_permission_based_on_destination_and_endpoint() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let executor_address = setup.blockchain.create_user_account(&rust_biguint!(5));
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
            sc.create_permission(
                managed_buffer!(b"addressAndEndpoint"),
                managed_biguint!(0),
                managed_address!(&action_receiver),
                managed_buffer!(b"myendpoint"),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_policy(
                managed_buffer!(b"builder"),
                managed_buffer!(b"addressAndEndpoint"),
                PolicyMethod::All,
                BigUint::from(0u64),
                10,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
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
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"addressAndEndpoint")]);

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b""),
                managed_buffer!(b""),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&executor_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            let proposal = sc.proposals(proposal_id).get();

            let actual = sc.get_actual_permissions(&proposal, &ManagedVec::from(actions));

            assert_eq!(1, actual.len());
        })
        .assert_ok();
}

#[test]
fn it_matches_a_permission_based_on_destination_and_endpoint_and_one_argument() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let executor_address = setup.blockchain.create_user_account(&rust_biguint!(5));
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));

            sc.create_permission(
                managed_buffer!(b"addressAndEndpoint"),
                managed_biguint!(0),
                managed_address!(&action_receiver),
                managed_buffer!(b"myendpoint"),
                ManagedVec::from(vec![managed_buffer!(b"testarg")]),
                ManagedVec::new(),
            );

            sc.create_policy(
                managed_buffer!(b"builder"),
                managed_buffer!(b"addressAndEndpoint"),
                PolicyMethod::All,
                BigUint::from(0u64),
                10,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
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
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"addressAndEndpoint")]);

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b""),
                managed_buffer!(b""),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&executor_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::from(vec![managed_buffer!(b"testarg")]),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            let proposal = sc.proposals(proposal_id).get();

            let actual = sc.get_actual_permissions(&proposal, &ManagedVec::from(actions));

            assert_eq!(1, actual.len());
        })
        .assert_ok();
}
