use elrond_wasm::elrond_codec::multi_types::OptionalValue;
use elrond_wasm::{types::*};
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::*;
use setup::*;

mod setup;

#[test]
fn it_marks_a_proposal_as_executed() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;

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
                votes_for: sc.quorum().get(),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(1).set(dummy_proposal);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.execute_endpoint(1, MultiValueManagedVec::new());

            let proposal = sc.proposals(1).get();
            assert_eq!(true, proposal.was_executed);
        })
        .assert_ok();
}

#[test]
fn it_fails_if_attempted_to_execute_again() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;

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
                votes_for: sc.quorum().get(),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(1).set(dummy_proposal);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.execute_endpoint(1, MultiValueManagedVec::new());

            sc.execute_endpoint(1, MultiValueManagedVec::new()); // and again
        })
        .assert_user_error("proposal is not executable");
}

#[test]
fn it_fails_if_the_proposal_is_still_active() {
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
                votes_for: sc.quorum().get(),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(1).set(dummy_proposal);

            sc.execute_endpoint(1, MultiValueManagedVec::new());
        })
        .assert_user_error("proposal is not executable");
}


#[test]
fn it_executes_actions_of_a_proposal() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.blockchain.set_egld_balance(setup.contract.address_ref(), &rust_biguint!(1000));

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(ENTITY_TOKEN_SUPPLY), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                address: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                token_id: managed_token_id!(b"EGLD"),
                token_nonce: 0,
                amount: managed_biguint!(5),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));

            sc.propose_endpoint(managed_buffer!(b"a"), managed_buffer!(b"b"), OptionalValue::Some(actions_hash.clone()));
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                address: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                token_id: managed_token_id!(b"EGLD"),
                token_nonce: 0,
                amount: managed_biguint!(5),
            });

            sc.execute_endpoint(1, MultiValueManagedVec::from(actions));
        })
        .assert_ok();

    setup.blockchain.check_egld_balance(&action_receiver, &rust_biguint!(5));
}

#[test]
fn it_fails_if_actions_to_execute_are_incongruent_to_actions_proposed() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.blockchain.set_egld_balance(setup.contract.address_ref(), &rust_biguint!(1000));

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(ENTITY_TOKEN_SUPPLY), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                address: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                token_id: managed_token_id!(b"EGLD"),
                token_nonce: 0,
                amount: managed_biguint!(5),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));

            sc.propose_endpoint(managed_buffer!(b"a"), managed_buffer!(b"b"), OptionalValue::Some(actions_hash.clone()));
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                address: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"yourendpoint"), // has changed from myendpoint to yourendpoint -> fail
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                token_id: managed_token_id!(b"EGLD"),
                token_nonce: 0,
                amount: managed_biguint!(5),
            });

            sc.execute_endpoint(1, MultiValueManagedVec::from(actions));
        })
        .assert_user_error("actions have been corrupted");
}

#[test]
fn it_executes_a_contract_call_action() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup
        .blockchain
        .set_esdt_balance(setup.contract.address_ref(), b"ACTION-123456", &rust_biguint!(1000));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let starts_at = 0u64;
            let voting_period_minutes = sc.voting_period_in_minutes().get() as u64;
            let ends_at = starts_at + voting_period_minutes * 60;
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                address: managed_address!(&action_receiver),
                token_id: managed_token_id!(b"ACTION-123456"),
                token_nonce: 0,
                amount: managed_biguint!(5),
                gas_limit: 5_000_000u64,
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::from(vec![managed_buffer!(b"arg1"), managed_buffer!(b"arg2")]),
            });

            let dummy_proposal = Proposal::<DebugApi> {
                actions_hash: sc.calculate_actions_hash(&ManagedVec::from(actions)),
                starts_at,
                ends_at,
                content_hash: managed_buffer!(b"hash"),
                id: 0,
                votes_against: managed_biguint!(0),
                votes_for: sc.quorum().get(),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(1).set(dummy_proposal);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                address: managed_address!(&action_receiver),
                token_id: managed_token_id!(b"ACTION-123456"),
                token_nonce: 0,
                amount: managed_biguint!(5),
                gas_limit: 5_000_000u64,
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::from(vec![managed_buffer!(b"arg1"), managed_buffer!(b"arg2")]),
            });

            sc.execute_endpoint(1, MultiValueManagedVec::from(actions));
        })
        .assert_ok();

    setup.blockchain.check_esdt_balance(&action_receiver, b"ACTION-123456", &rust_biguint!(5));
}

#[test]
fn it_fails_to_spend_vote_tokens() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    // set available balance to 5
    setup
        .blockchain
        .set_esdt_balance(setup.contract.address_ref(), ENTITY_TOKEN_ID, &rust_biguint!(5));

    // but try to spend 6 with a proposal action
    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let starts_at = 0u64;
            let voting_period_minutes = sc.voting_period_in_minutes().get() as u64;
            let ends_at = starts_at + voting_period_minutes * 60;
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                address: managed_address!(&action_receiver),
                token_id: managed_token_id!(ENTITY_TOKEN_ID),
                token_nonce: 0,
                amount: managed_biguint!(6),
                gas_limit: 5_000_000u64,
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::from(vec![managed_buffer!(b"arg1")]),
            });

            let proposal = Proposal::<DebugApi> {
                actions_hash: sc.calculate_actions_hash(&ManagedVec::from(actions)),
                starts_at,
                ends_at,
                content_hash: managed_buffer!(b"hash"),
                id: 0,
                votes_against: managed_biguint!(0),
                votes_for: sc.quorum().get(),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(1).set(proposal);
        })
        .assert_ok();

    // add to the sc token balance: vote for with 100 tokens
    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(100), |sc| {
            sc.vote_for_endpoint(1);
        })
        .assert_ok();

    // add to the sc token balance: vote against with 100 tokens
    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(20), |sc| {
            sc.vote_against_endpoint(1);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    // but it should FAIL because vote tokens should NOT be spendable
    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                address: managed_address!(&action_receiver),
                token_id: managed_token_id!(ENTITY_TOKEN_ID),
                token_nonce: 0,
                amount: managed_biguint!(6),
                gas_limit: 5_000_000u64,
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::from(vec![managed_buffer!(b"arg1")]),
            });

            sc.execute_endpoint(1, MultiValueManagedVec::from(actions));
        })
        .assert_user_error("not enough governance tokens available");
}
