use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::*;
use entity::permission::*;
use setup::*;

mod setup;

#[test]
fn it_creates_a_proposal() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let owner_address = setup.owner_address.clone();
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    setup.blockchain.set_nft_balance(&owner_address, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(5), &0);

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(3), |sc| {
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
        .execute_query(&setup.contract, |sc| {
            let proposal = sc.proposals(proposal_id).get();

            // proposal
            assert_eq!(1, proposal.id);
            assert_eq!(managed_address!(&owner_address), proposal.proposer);
            assert_eq!(managed_buffer!(b"content hash"), proposal.content_hash);
            assert_eq!(managed_buffer!(b""), proposal.actions_hash);
            assert_eq!(false, proposal.was_executed);
            assert_eq!(managed_biguint!(3), proposal.votes_for);
            assert_eq!(managed_biguint!(0), proposal.votes_against);

            // storage
            assert_eq!(2, sc.next_proposal_id().get());
            assert_eq!(managed_biguint!(3), sc.votes(proposal.id, &managed_address!(&owner_address)).get());
            assert_eq!(managed_biguint!(3), sc.protected_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID)).get());
        })
        .assert_ok();
}

#[test]
fn it_creates_a_proposal_with_poll() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let owner_address = setup.owner_address.clone();

    setup.configure_gov_token(true);

    setup.blockchain.set_nft_balance(&owner_address, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(5), &0);

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(2), |sc| {
            let poll_option_id = 2u8;

            let proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b"content hash"),
                managed_buffer!(b"content signature"),
                managed_buffer!(b""),
                poll_option_id,
                MultiValueManagedVec::new(),
            );

            assert_eq!(managed_biguint!(2), sc.proposal_poll(proposal_id, poll_option_id).get());
        })
        .assert_ok();
}
