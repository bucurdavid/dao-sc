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

    setup.configure_gov_token();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL), |sc| {
        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), managed_buffer!(b""), MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
        sc.vote_for_endpoint(proposal_id);

        let proposal = sc.proposals(proposal_id).get();

        assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL + 25), proposal.votes_for);
        assert_eq!(managed_biguint!(0), proposal.votes_against);
        assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL + 25), sc.protected_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID)).get());
        assert_eq!(managed_biguint!(25), sc.votes(proposal.id, &managed_address!(&voter_address)).get());
        assert!(sc.withdrawable_proposal_ids(&managed_address!(&voter_address)).contains(&proposal.id));
    })
    .assert_ok();

    // same vote again to assert it adds up
    setup.blockchain.execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
        sc.vote_for_endpoint(proposal_id);

        let proposal = sc.proposals(proposal_id).get();

        assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL + 50), proposal.votes_for);
        assert_eq!(managed_biguint!(0), proposal.votes_against);
        assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL + 50), sc.protected_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID)).get());
        assert_eq!(managed_biguint!(50), sc.votes(proposal.id, &managed_address!(&voter_address)).get());
    })
    .assert_ok();
}

#[test]
fn it_votes_against_a_proposal() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voter_address = setup.user_address.clone();
    let mut proposal_id = 0;

    setup.configure_gov_token();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL), |sc| {
        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), managed_buffer!(b""), MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
        sc.vote_against_endpoint(proposal_id);

        let proposal = sc.proposals(proposal_id).get();

        assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL), proposal.votes_for);
        assert_eq!(managed_biguint!(25), proposal.votes_against);
        assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL + 25), sc.protected_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID)).get());
        assert_eq!(managed_biguint!(25), sc.votes(proposal.id, &managed_address!(&voter_address)).get());
        assert!(sc.withdrawable_proposal_ids(&managed_address!(&voter_address)).contains(&proposal.id));
    })
    .assert_ok();

    // same vote again to assert it adds up
    setup.blockchain.execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
        sc.vote_against_endpoint(proposal_id);

        let proposal = sc.proposals(proposal_id).get();

        assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL), proposal.votes_for);
        assert_eq!(managed_biguint!(50), proposal.votes_against);
        assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL + 50), sc.protected_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID)).get());
        assert_eq!(managed_biguint!(50), sc.votes(proposal.id, &managed_address!(&voter_address)).get());
    })
    .assert_ok();
}

#[test]
fn it_fails_if_proposal_voting_period_has_ended() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let mut proposal_id = 0;

    setup.configure_gov_token();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL), |sc| {
        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), managed_buffer!(b""), MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
        sc.vote_against_endpoint(proposal_id);
    })
    .assert_user_error("proposal is not active");
}
