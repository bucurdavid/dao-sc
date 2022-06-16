use elrond_wasm::elrond_codec::multi_types::OptionalValue;
use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::vote::*;
use entity::governance::*;
use setup::*;

mod setup;

#[test]
fn it_creates_a_proposal() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let owner_address = setup.owner_address.clone();

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.owner_address,
            &setup.contract,
            ENTITY_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
            |sc| {
                sc.propose_endpoint(
                    managed_buffer!(b"content hash"),
                    managed_buffer!(b"content signature"),
                    OptionalValue::None,
                );
            },
        )
        .assert_ok();

    // assert contract storage
    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let proposal = sc.proposals(1).get();

            assert_eq!(1, proposal.id);
            assert_eq!(managed_address!(&owner_address), proposal.proposer);
            assert_eq!(managed_buffer!(b"content hash"), proposal.content_hash);
            assert_eq!(managed_buffer!(b""), proposal.actions_hash);
            assert_eq!(false, proposal.was_executed);
            assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL), proposal.votes_for);
            assert_eq!(managed_biguint!(0), proposal.votes_against);

            assert_eq!(2, sc.next_proposal_id().get());
        })
        .assert_ok();
}

#[test]
fn it_creates_a_proposal_with_actions() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.owner_address,
            &setup.contract,
            ENTITY_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
            |sc| {
                sc.propose_endpoint(
                    managed_buffer!(b"content hash"),
                    managed_buffer!(b"content signature"),
                    OptionalValue::Some(managed_buffer!(b"actions hash")),
                );
            },
        )
        .assert_ok();

    // assert contract storage
    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let proposal = sc.proposals(1).get();

            assert_eq!(managed_buffer!(b"actions hash"), proposal.actions_hash);
        })
        .assert_ok();
}

#[test]
fn it_sends_a_vote_nft_to_the_voter() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let owner_address = setup.owner_address.clone();

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.owner_address,
            &setup.contract,
            ENTITY_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
            |sc| {
                sc.propose_endpoint(
                    managed_buffer!(b"content hash"),
                    managed_buffer!(b"content signature"),
                    OptionalValue::None,
                );
            },
        )
        .assert_ok();

    // assert voter received vote nft
    setup.blockchain.execute_in_managed_environment(|| {
        setup.blockchain.check_nft_balance(
            &setup.owner_address,
            VOTE_NFT_TOKEN_ID,
            1,
            &rust_biguint!(1),
            Some(&VoteNFTAttributes::<DebugApi> {
                proposal_id: 1,
                vote_type: VoteType::For,
                vote_weight: managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
                voter: managed_address!(&owner_address),
                payment: EsdtTokenPayment::new(managed_token_id!(ENTITY_TOKEN_ID), 0, managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL)),
            }),
        );
    });
}

#[test]
fn it_fails_if_bad_token() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.user_address,
            &setup.contract,
            ENTITY_FAKE_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
            |sc| {
                sc.propose_endpoint(
                    managed_buffer!(b""),
                    managed_buffer!(b""),
                    OptionalValue::None,
                );
            },
        )
        .assert_user_error("invalid token");
}

#[test]
fn it_fails_if_bad_vote_weight_amount() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.user_address,
            &setup.contract,
            ENTITY_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL - 1),
            |sc| {
                sc.propose_endpoint(
                    managed_buffer!(b""),
                    managed_buffer!(b""),
                    OptionalValue::None,
                );
            },
        )
        .assert_user_error("insufficient vote weight");
}
