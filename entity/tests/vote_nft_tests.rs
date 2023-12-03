use entity::config::*;
use entity::governance::errors::*;
use entity::governance::*;
use multiversx_sc::codec::multi_types::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_votes_for_a_proposal() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voter_address = setup.user_address.clone();
    let mut proposal_id = 0;

    setup.configure_gov_token(false);

    setup
        .blockchain
        .set_nft_balance(&setup.owner_address, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(1), &0u32);

    setup.blockchain.set_nft_balance(&voter_address, ENTITY_GOV_TOKEN_ID, 2, &rust_biguint!(1), &0u32);

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(1), |sc| {
            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                POLL_DEFAULT_ID,
                MultiValueManagedVec::new(),
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 2, &rust_biguint!(1), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);

            let proposal = sc.proposals(proposal_id).get();

            assert_eq!(managed_biguint!(2), proposal.votes_for);
            assert_eq!(managed_biguint!(0), proposal.votes_against);
            assert_eq!(managed_biguint!(0), sc.guarded_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID), 0).get());
            assert_eq!(0, sc.withdrawable_proposal_ids(&managed_address!(&voter_address)).len());
            assert!(sc.withdrawable_votes(proposal.id, &managed_address!(&voter_address)).is_empty());
        })
        .assert_ok();
}

#[test]
fn it_votes_for_a_proposal_with_poll() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voter_address = setup.user_address.clone();
    let mut proposal_id = 0;
    let poll_option_id = 2u8;

    setup.configure_gov_token(false);

    setup
        .blockchain
        .set_nft_balance(&setup.owner_address, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(1), &0u32);

    setup.blockchain.set_nft_balance(&voter_address, ENTITY_GOV_TOKEN_ID, 2, &rust_biguint!(1), &0u32);

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(1), |sc| {
            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                poll_option_id,
                MultiValueManagedVec::new(),
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 2, &rust_biguint!(1), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::Some(poll_option_id));

            assert_eq!(managed_biguint!(2), sc.proposal_poll(proposal_id, poll_option_id).get());
        })
        .assert_ok();
}

#[test]
fn it_votes_against_a_proposal() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voter_address = setup.user_address.clone();
    let mut proposal_id = 0;

    setup.configure_gov_token(false);

    setup
        .blockchain
        .set_nft_balance(&setup.owner_address, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(1), &0u32);
    setup.blockchain.set_nft_balance(&voter_address, ENTITY_GOV_TOKEN_ID, 2, &rust_biguint!(1), &0u32);

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(1), |sc| {
            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                POLL_DEFAULT_ID,
                MultiValueManagedVec::new(),
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 2, &rust_biguint!(1), |sc| {
            sc.vote_against_endpoint(proposal_id, OptionalValue::None);

            let proposal = sc.proposals(proposal_id).get();

            assert_eq!(managed_biguint!(1), proposal.votes_for);
            assert_eq!(managed_biguint!(1), proposal.votes_against);
            assert_eq!(managed_biguint!(0), sc.guarded_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID), 0).get());
            assert_eq!(0, sc.withdrawable_proposal_ids(&managed_address!(&voter_address)).len());
            assert!(sc.withdrawable_votes(proposal.id, &managed_address!(&voter_address)).is_empty());
        })
        .assert_ok();
}

#[test]
fn it_sends_the_nfts_back() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let owner_address = setup.owner_address.clone();
    let voter_address = setup.user_address.clone();
    let mut proposal_id = 0;

    setup.configure_gov_token(false);

    setup.blockchain.set_nft_balance(&owner_address, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(1), &0u32);
    setup.blockchain.set_nft_balance(&voter_address, ENTITY_GOV_TOKEN_ID, 2, &rust_biguint!(1), &0u32);

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(1), |sc| {
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

    setup
        .blockchain
        .execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 2, &rust_biguint!(1), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);

            let proposal = sc.proposals(proposal_id).get();

            assert_eq!(managed_biguint!(2), proposal.votes_for);
            assert_eq!(managed_biguint!(0), proposal.votes_against);
            assert_eq!(managed_biguint!(0), sc.guarded_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID), 0).get());
            assert_eq!(0, sc.withdrawable_proposal_ids(&managed_address!(&voter_address)).len());
            assert!(sc.withdrawable_votes(proposal.id, &managed_address!(&voter_address)).is_empty());
        })
        .assert_ok();

    setup
        .blockchain
        .check_nft_balance::<u32>(&voter_address, ENTITY_GOV_TOKEN_ID, 2, &rust_biguint!(1), Option::None);
}

#[test]
fn it_fails_to_vote_twice_with_the_same_nft() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let owner_address = setup.owner_address.clone();
    let voter_address = setup.user_address.clone();
    let mut proposal_id = 0;

    setup.configure_gov_token(false);

    setup.blockchain.set_nft_balance(&owner_address, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(1), &0u32);
    setup.blockchain.set_nft_balance(&voter_address, ENTITY_GOV_TOKEN_ID, 2, &rust_biguint!(1), &0u32);

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(1), |sc| {
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

    setup
        .blockchain
        .execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 2, &rust_biguint!(1), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 2, &rust_biguint!(1), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_user_error(&String::from_utf8(ALREADY_VOTED_WITH_TOKEN.to_vec()).unwrap());
}

#[test]
fn it_fails_when_less_than_configured_min_vote_weight() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let owner_address = setup.owner_address.clone();
    let voter_address = setup.user_address.clone();
    let mut proposal_id = 0;

    setup.configure_gov_token(false);

    setup.blockchain.set_nft_balance(&owner_address, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(1), &0u32);
    setup.blockchain.set_nft_balance(&voter_address, ENTITY_GOV_TOKEN_ID, 2, &rust_biguint!(1), &0u32);

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(1), |sc| {
            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b"content hash"),
                managed_buffer!(b"content signature"),
                ManagedBuffer::new(),
                POLL_DEFAULT_ID,
                MultiValueManagedVec::new(),
            );

            sc.try_change_min_vote_weight(managed_biguint!(3));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 2, &rust_biguint!(1), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_user_error("not enought vote weight");
}

#[test]
fn it_fails_when_vote_token_has_restricted_nonce() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let owner_address = setup.owner_address.clone();
    let voter_address = setup.user_address.clone();
    let allowed_sft_nonce = 1;
    let invalid_sft_nonce = 2;
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .set_nft_balance(&owner_address, ENTITY_GOV_TOKEN_ID, allowed_sft_nonce, &rust_biguint!(5), &0);

    setup
        .blockchain
        .set_nft_balance(&voter_address, ENTITY_GOV_TOKEN_ID, invalid_sft_nonce, &rust_biguint!(5), &0);

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
            allowed_sft_nonce,
            &rust_biguint!(MIN_PROPOSE_WEIGHT),
            |sc| {
                proposal_id = sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    managed_buffer!(b"content hash"),
                    managed_buffer!(b"content signature"),
                    ManagedBuffer::new(),
                    POLL_DEFAULT_ID,
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_ok();

    // assert - voteFor
    setup
        .blockchain
        .execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, invalid_sft_nonce, &rust_biguint!(1), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_user_error("vote token nonce is restricted");

    // assert - voteAgainst
    setup
        .blockchain
        .execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, invalid_sft_nonce, &rust_biguint!(1), |sc| {
            sc.vote_against_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_user_error("vote token nonce is restricted");
}
