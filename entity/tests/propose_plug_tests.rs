use entity::config::*;
use entity::governance::*;
use entity::permission::*;
use entity::plug::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_creates_a_proposal_using_the_plugs_weight() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let proposal_id = 1;

    setup.configure_plug(100, 50);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
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
            assert_eq!(managed_biguint!(100), proposal.votes_for);

            let user_id = sc.users().get_user_id(&managed_address!(&proposer_address));
            assert!(sc.plug_votes(proposal_id).contains(&user_id));

            // not withdrawable
            assert!(!sc.withdrawable_proposal_ids(&managed_address!(&proposer_address)).contains(&proposal.id));
        })
        .assert_ok();
}
