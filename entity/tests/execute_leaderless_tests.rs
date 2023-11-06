use entity::config::*;
use entity::governance::errors::*;
use entity::governance::proposal::*;
use entity::governance::*;
use multiversx_sc::codec::multi_types::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_executes_actions_of_a_succeeded_proposal() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.configure_gov_token(true);
    setup.configure_leaderless();

    setup.blockchain.set_egld_balance(setup.contract.address_ref(), &rust_biguint!(1000));

    setup
        .blockchain
        .execute_esdt_transfer(
            &proposer_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(ENTITY_GOV_TOKEN_SUPPLY),
            |sc| {
                let mut actions = Vec::<Action<DebugApi>>::new();

                actions.push(Action::<DebugApi> {
                    destination: managed_address!(&action_receiver),
                    endpoint: managed_buffer!(b"myendpoint"),
                    arguments: ManagedVec::new(),
                    gas_limit: 5_000_000u64,
                    value: managed_biguint!(5),
                    payments: ManagedVec::new(),
                });

                let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
                let actions_permissions = MultiValueManagedVec::new();

                sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    managed_buffer!(b"a"),
                    managed_buffer!(b"b"),
                    actions_hash,
                    POLL_DEFAULT_ID,
                    actions_permissions,
                );
            },
        )
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(5),
                payments: ManagedVec::new(),
            });

            sc.execute_endpoint(1, MultiValueManagedVec::from(actions));
        })
        .assert_ok();

    setup.blockchain.check_egld_balance(&action_receiver, &rust_biguint!(5));
}

#[test]
fn it_fails_when_the_proposal_has_been_defeated() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.configure_gov_token(true);
    setup.configure_leaderless();

    // proposing with minimum to propose which is less than required quorum
    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(MIN_PROPOSE_WEIGHT), |sc| {
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

            proposal_id = sc.propose_endpoint(
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

            sc.execute_endpoint(proposal_id, MultiValueManagedVec::from(actions));
        })
        .assert_user_error("no permission for action");
}

#[test]
fn it_fails_when_actions_to_execute_are_incongruent_to_actions_proposed() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.configure_gov_token(true);
    setup.configure_leaderless();

    setup.blockchain.set_egld_balance(setup.contract.address_ref(), &rust_biguint!(1000));

    setup
        .blockchain
        .execute_esdt_transfer(
            &proposer_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(ENTITY_GOV_TOKEN_SUPPLY),
            |sc| {
                let mut actions = Vec::<Action<DebugApi>>::new();

                actions.push(Action::<DebugApi> {
                    destination: managed_address!(&action_receiver),
                    endpoint: managed_buffer!(b"myendpoint"),
                    arguments: ManagedVec::new(),
                    gas_limit: 5_000_000u64,
                    value: managed_biguint!(5),
                    payments: ManagedVec::new(),
                });

                let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
                let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"perm")]);

                sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    managed_buffer!(b"a"),
                    managed_buffer!(b"b"),
                    actions_hash,
                    POLL_DEFAULT_ID,
                    actions_permissions,
                );
            },
        )
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"yourendpoint"), // has changed from myendpoint to yourendpoint -> fail
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(5),
                payments: ManagedVec::new(),
            });

            sc.execute_endpoint(1, MultiValueManagedVec::from(actions));
        })
        .assert_user_error("actions have been corrupted");
}

#[test]
fn it_fails_to_spend_esdt_vote_tokens() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.configure_gov_token(true);
    setup.configure_leaderless();

    // set available balance to 5
    setup
        .blockchain
        .set_esdt_balance(setup.contract.address_ref(), ENTITY_GOV_TOKEN_ID, &rust_biguint!(5));

    // but try to spend 6 with a proposal action
    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                gas_limit: 5_000_000u64,
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::from(vec![managed_buffer!(b"arg1")]),
                value: managed_biguint!(0),
                payments: ManagedVec::from(vec![EsdtTokenPayment::new(managed_token_id!(ENTITY_GOV_TOKEN_ID), 0, managed_biguint!(6))]),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
            let actions_permissions = MultiValueManagedVec::new();

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b"a"),
                managed_buffer!(b"b"),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    // add to the sc token balance: vote for with 100 tokens
    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(100), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    // add to the sc token balance: vote against with 100 tokens
    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(20), |sc| {
            sc.vote_against_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    // but it should FAIL because vote tokens should NOT be spendable
    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                gas_limit: 5_000_000u64,
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::from(vec![managed_buffer!(b"arg1")]),
                value: managed_biguint!(0),
                payments: ManagedVec::from(vec![EsdtTokenPayment::new(managed_token_id!(ENTITY_GOV_TOKEN_ID), 0, managed_biguint!(6))]),
            });

            sc.execute_endpoint(proposal_id, MultiValueManagedVec::from(actions));
        })
        .assert_user_error(&String::from_utf8(NOT_ENOUGH_GOV_TOKENS_AVAILABLE.to_vec()).unwrap());
}
