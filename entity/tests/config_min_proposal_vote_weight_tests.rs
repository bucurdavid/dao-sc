use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::*;
use setup::*;

mod setup;

#[test]
fn it_changes_the_min_proposal_vote_weight_if_contract_calls_itself() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup.configure_gov_token();

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.change_min_proposal_vote_weight_endpoint(managed_biguint!(1000));

            assert_eq!(sc.min_proposal_vote_weight().get(), managed_biguint!(1000));
        })
        .assert_ok();
}

#[test]
fn it_fails_if_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.change_min_proposal_vote_weight_endpoint(managed_biguint!(1000));
        })
        .assert_user_error("action not allowed by user");
}

#[test]
fn it_fails_if_gov_token_is_not_set() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.change_min_proposal_vote_weight_endpoint(managed_biguint!(1000));
        })
        .assert_user_error("gov token must be set");
}
