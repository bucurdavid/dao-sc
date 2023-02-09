use entity::config::*;
use entity::governance::*;
use entity::plug::*;
use multiversx_sc::codec::multi_types::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_votes_for_a_proposal_using_the_plugs_weight() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voter_address = setup.user_address.clone();
    let proposal_id = 1;

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

    // vote for
    setup
        .blockchain
        .execute_tx(&voter_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    // assert
    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let proposal = sc.proposals(proposal_id).get();

            assert_eq!(managed_biguint!(200), proposal.votes_for); // 100 from proposer + 100 from voter
            assert_eq!(managed_biguint!(0), proposal.votes_against);

            let user_id = sc.users().get_user_id(&managed_address!(&voter_address));
            assert!(sc.plug_votes(proposal_id).contains(&user_id));

            // not withdrawable
            assert!(!sc.withdrawable_proposal_ids(&managed_address!(&voter_address)).contains(&proposal.id));
        })
        .assert_ok();
}

#[test]
fn it_fails_to_vote_twice() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voter_address = setup.user_address.clone();
    let proposal_id = 1;

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

    // vote for
    setup
        .blockchain
        .execute_tx(&voter_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    // vote for again - should fail
    let _ = setup.blockchain.execute_tx(&voter_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.vote_for_endpoint(proposal_id, OptionalValue::None);
    });

    // assert
    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let proposal = sc.proposals(proposal_id).get();

            assert_eq!(managed_biguint!(200), proposal.votes_for); // 100 from proposer + 100 from voter (only once)
        })
        .assert_ok();
}
