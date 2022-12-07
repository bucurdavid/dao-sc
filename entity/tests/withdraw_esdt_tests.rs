use elrond_wasm::elrond_codec::multi_types::*;
use elrond_wasm::types::MultiValueManagedVec;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::GovernanceModule;
use setup::*;

mod setup;

#[test]
fn it_withdraws_tokens_used_for_voting() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address.clone();
    let mut proposal_id = 0u64;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_esdt_transfer(
            &user_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
            |sc| {
                proposal_id = sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    managed_buffer!(b"content hash"),
                    managed_buffer!(b"content signature"),
                    managed_buffer!(b""),
                    POLL_DEFAULT_ID,
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&user_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&user_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.vote_against_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.withdraw_endpoint();
        })
        .assert_ok();

    setup
        .blockchain
        .check_esdt_balance(&user_address, ENTITY_GOV_TOKEN_ID, &rust_biguint!(ENTITY_GOV_TOKEN_SUPPLY));
}

#[test]
fn it_clears_the_voters_withdrawable_storage_for_the_proposal() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voter_address = setup.user_address.clone();
    let mut proposal_id = 0u64;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_esdt_transfer(
            &voter_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
            |sc| {
                proposal_id = sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    managed_buffer!(b"content hash"),
                    managed_buffer!(b"content signature"),
                    managed_buffer!(b""),
                    POLL_DEFAULT_ID,
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_tx(&voter_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.withdraw_endpoint();
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(
                managed_biguint!(0),
                sc.withdrawable_votes(proposal_id, &managed_address!(&voter_address), &managed_token_id!(ENTITY_GOV_TOKEN_ID), 0)
                    .get()
            );
            assert!(!sc.withdrawable_proposal_ids(&managed_address!(&voter_address)).contains(&proposal_id));
            assert!(!sc.withdrawable_proposal_token_nonces(proposal_id, &managed_address!(&voter_address)).contains(&0));
        })
        .assert_ok();
}

#[test]
fn it_reduces_the_protected_vote_token_amount() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voter_address = setup.user_address.clone();
    let mut proposal_id = 0u64;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_esdt_transfer(
            &voter_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
            |sc| {
                proposal_id = sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    managed_buffer!(b"content hash"),
                    managed_buffer!(b"content signature"),
                    managed_buffer!(b""),
                    POLL_DEFAULT_ID,
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_tx(&voter_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.withdraw_endpoint();
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(managed_biguint!(0), sc.guarded_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID), 0).get());
        })
        .assert_ok();
}

#[test]
fn it_does_not_withdraw_tokens_from_proposals_that_are_still_active() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address.clone();
    let mut proposal_id = 0u64;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_esdt_transfer(
            &user_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
            |sc| {
                proposal_id = sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    managed_buffer!(b"content hash"),
                    managed_buffer!(b"content signature"),
                    managed_buffer!(b""),
                    POLL_DEFAULT_ID,
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.withdraw_endpoint();
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(
                managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
                sc.withdrawable_votes(proposal_id, &managed_address!(&user_address), &managed_token_id!(ENTITY_GOV_TOKEN_ID), 0)
                    .get()
            );
            assert!(sc.withdrawable_proposal_ids(&managed_address!(&user_address)).contains(&proposal_id));
        })
        .assert_ok();
}
