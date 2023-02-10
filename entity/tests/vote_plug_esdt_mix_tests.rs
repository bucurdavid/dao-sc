use entity::config::*;
use entity::governance::*;
use multiversx_sc::codec::multi_types::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_combines_vote_weights_from_plug_and_esdts() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voter_address = setup.user_address.clone();
    let proposal_id = 1;
    let plug_weight = 100;

    setup.configure_gov_token(true);
    setup.configure_plug(100, 50);

    // propose as any user
    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
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

    // vote for - using plug and esdts from payment
    setup
        .blockchain
        .execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(50), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    // assert
    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let proposal = sc.proposals(proposal_id).get();

            assert_eq!(managed_biguint!(plug_weight + plug_weight + 50), proposal.votes_for); // 100 from proposer + 100 from voter plug + 50 from voter esdt

            // has withdrawable esdts
            assert!(sc.withdrawable_proposal_ids(&managed_address!(&voter_address)).contains(&proposal.id));
            assert_eq!(
                sc.withdrawable_votes(proposal.id, &managed_address!(&voter_address)).get(1).amount,
                managed_biguint!(50)
            );
        })
        .assert_ok();
}
