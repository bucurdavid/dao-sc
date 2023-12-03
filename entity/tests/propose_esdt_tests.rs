use entity::config::*;
use entity::governance::proposal::{Action, ProposalModule};
use entity::governance::*;
use entity::permission::{PermissionModule, ROLE_BUILTIN_LEADER};
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_creates_a_proposal() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let owner_address = setup.owner_address.clone();
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_esdt_transfer(&owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(MIN_PROPOSE_WEIGHT), |sc| {
            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b"content hash"),
                managed_buffer!(b"content signature"),
                ManagedBuffer::new(),
                POLL_DEFAULT_ID,
                MultiValueManagedVec::new(),
            );
        })
        .assert_ok();

    // assert
    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let proposal = sc.proposals(proposal_id).get();

            // proposal
            assert_eq!(1, proposal.id);
            assert_eq!(managed_address!(&owner_address), proposal.proposer);
            assert_eq!(managed_buffer!(b"content hash"), proposal.content_hash);
            assert_eq!(ManagedBuffer::new(), proposal.actions_hash);
            assert_eq!(false, proposal.was_executed);
            assert_eq!(managed_biguint!(MIN_PROPOSE_WEIGHT), proposal.votes_for);
            assert_eq!(managed_biguint!(0), proposal.votes_against);

            // storage
            assert_eq!(2, sc.next_proposal_id().get());

            let withdrawable_mapper = sc.withdrawable_votes(proposal.id, &managed_address!(&owner_address)).get(1);
            assert_eq!(managed_token_id!(ENTITY_GOV_TOKEN_ID), withdrawable_mapper.token_identifier);
            assert_eq!(managed_biguint!(MIN_PROPOSE_WEIGHT), withdrawable_mapper.amount);

            assert_eq!(
                managed_biguint!(MIN_PROPOSE_WEIGHT),
                sc.guarded_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID), 0).get()
            );
            assert!(sc.withdrawable_proposal_ids(&managed_address!(&owner_address)).contains(&proposal.id));
        })
        .assert_ok();
}

#[test]
fn it_creates_a_proposal_with_poll() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.owner_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(MIN_PROPOSE_WEIGHT),
            |sc| {
                let poll_option_id = 2u8;

                let proposal_id = sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    managed_buffer!(b"content hash"),
                    managed_buffer!(b"content signature"),
                    ManagedBuffer::new(),
                    poll_option_id,
                    MultiValueManagedVec::new(),
                );

                assert_eq!(managed_biguint!(MIN_PROPOSE_WEIGHT), sc.proposal_poll(proposal_id, poll_option_id).get());
            },
        )
        .assert_ok();
}

#[test]
fn it_creates_a_proposal_with_actions() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
        })
        .assert_ok();

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
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"any")]);

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b"content hash"),
                managed_buffer!(b"content signature"),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            let expected = sc.calculate_actions_hash(&ManagedVec::from(actions));

            let proposal = sc.proposals(proposal_id).get();

            assert_eq!(expected, proposal.actions_hash);
        })
        .assert_ok();
}

#[test]
fn it_fails_when_bad_token() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.user_address,
            &setup.contract,
            ENTITY_FAKE_TOKEN_ID,
            0,
            &rust_biguint!(MIN_PROPOSE_WEIGHT),
            |sc| {
                sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    POLL_DEFAULT_ID,
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_user_error("invalid payment token");
}

#[test]
fn it_fails_when_bad_vote_weight_amount() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.user_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(MIN_PROPOSE_WEIGHT - 1),
            |sc| {
                sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    POLL_DEFAULT_ID,
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_user_error("insufficient vote weight");
}

#[test]
fn it_fails_when_trusted_host_id_is_already_known() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.user_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(MIN_PROPOSE_WEIGHT),
            |sc| {
                sc.propose_endpoint(
                    managed_buffer!(b"thesame"),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    POLL_DEFAULT_ID,
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.user_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(MIN_PROPOSE_WEIGHT),
            |sc| {
                sc.propose_endpoint(
                    managed_buffer!(b"thesame"),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    POLL_DEFAULT_ID,
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_user_error("proposal already registered");
}

#[test]
fn it_fails_when_vote_token_has_restricted_nonce() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let owner_address = setup.owner_address.clone();
    let allowed_sft_nonce = 1;
    let invalid_sft_nonce = 2;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .set_nft_balance(&owner_address, ENTITY_GOV_TOKEN_ID, invalid_sft_nonce, &rust_biguint!(5), &0);

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            let mut restricted_nonces = MultiValueEncoded::new();
            restricted_nonces.push(allowed_sft_nonce);
            sc.set_restricted_vote_nonces_endpoint(restricted_nonces);
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(
            &owner_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            invalid_sft_nonce,
            &rust_biguint!(MIN_PROPOSE_WEIGHT),
            |sc| {
                sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    managed_buffer!(b"content hash"),
                    managed_buffer!(b"content signature"),
                    ManagedBuffer::new(),
                    POLL_DEFAULT_ID,
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_user_error("vote token nonce is restricted");
}
