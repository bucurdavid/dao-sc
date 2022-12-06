use elrond_wasm::elrond_codec::multi_types::*;
use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::*;
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
                managed_buffer!(b""),
                managed_buffer!(b""),
                managed_buffer!(b""),
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
            assert_eq!(managed_biguint!(0), sc.protected_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID)).get());
            assert_eq!(managed_biguint!(0), sc.votes(proposal.id, &managed_address!(&voter_address)).get());
            assert_eq!(0, sc.withdrawable_proposal_ids(&managed_address!(&voter_address)).len());
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
                managed_buffer!(b""),
                managed_buffer!(b""),
                managed_buffer!(b""),
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
                managed_buffer!(b""),
                managed_buffer!(b""),
                managed_buffer!(b""),
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
            assert_eq!(managed_biguint!(0), sc.protected_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID)).get());
            assert_eq!(managed_biguint!(0), sc.votes(proposal.id, &managed_address!(&voter_address)).get());
            assert_eq!(0, sc.withdrawable_proposal_ids(&managed_address!(&voter_address)).len());
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
                managed_buffer!(b""),
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
            assert_eq!(managed_biguint!(0), sc.protected_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID)).get());
            assert_eq!(managed_biguint!(0), sc.votes(proposal.id, &managed_address!(&voter_address)).get());
            assert_eq!(0, sc.withdrawable_proposal_ids(&managed_address!(&voter_address)).len());
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
                managed_buffer!(b""),
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
        .assert_user_error("already voted with nft");
}

#[test]
fn it_fails_if_less_than_configured_min_vote_weight() {
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
                managed_buffer!(b""),
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
