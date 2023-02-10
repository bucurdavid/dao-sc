use entity::config::*;
use entity::governance::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_combines_propose_weights_from_plug_and_esdts() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let proposal_id = 1;
    let plug_weight = 100;

    setup.configure_gov_token(true);
    setup.configure_plug(100, 50);

    // propose
    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(MIN_PROPOSE_WEIGHT), |sc| {
            sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
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

            // check proposal
            assert_eq!(1, proposal.id);
            assert_eq!(managed_address!(&proposer_address), proposal.proposer);
            assert_eq!(ManagedBuffer::new(), proposal.actions_hash);
            assert_eq!(false, proposal.was_executed);
            assert_eq!(managed_biguint!(MIN_PROPOSE_WEIGHT + plug_weight), proposal.votes_for);
            assert_eq!(managed_biguint!(0), proposal.votes_against);
            assert_eq!(2, sc.next_proposal_id().get());

            // check withdrawable
            assert!(sc.withdrawable_proposal_ids(&managed_address!(&proposer_address)).contains(&proposal.id));

            let withdrawable_mapper = sc.withdrawable_votes(proposal.id, &managed_address!(&proposer_address)).get(1);
            assert_eq!(managed_token_id!(ENTITY_GOV_TOKEN_ID), withdrawable_mapper.token_identifier);
            assert_eq!(managed_biguint!(MIN_PROPOSE_WEIGHT), withdrawable_mapper.amount);

            // check guarded tokens
            assert_eq!(
                managed_biguint!(MIN_PROPOSE_WEIGHT),
                sc.guarded_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID), 0).get()
            );
            assert!(sc.withdrawable_proposal_ids(&managed_address!(&proposer_address)).contains(&proposal.id));
        })
        .assert_ok();
}
