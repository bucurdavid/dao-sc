use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::*;

mod setup;

#[test]
fn it_changes_min_proposal_vote_weight() {
    let mut setup = setup::setup_entity(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.change_min_proposal_vote_weight_endpoint(managed_biguint!(1000));

            assert_eq!(sc.min_proposal_vote_weight().get(), managed_biguint!(1000));
        })
        .assert_ok();
}

#[test]
fn it_changes_the_min_proposal_vote_weight_if_entity_is_sealed_but_contract_calls_itself() {
    let mut setup = setup::setup_entity(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.sealed().set(SEALED_ON);

            sc.change_min_proposal_vote_weight_endpoint(managed_biguint!(1000));

            assert_eq!(sc.min_proposal_vote_weight().get(), managed_biguint!(1000));
        })
        .assert_ok();
}

#[test]
fn it_fails_if_the_entity_is_sealed() {
    let mut setup = setup::setup_entity(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.sealed().set(SEALED_ON);

            sc.change_min_proposal_vote_weight_endpoint(managed_biguint!(1000));
        })
        .assert_user_error("action not allowed by user");
}
