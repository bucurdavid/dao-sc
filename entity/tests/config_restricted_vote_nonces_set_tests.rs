use entity::config::*;
use entity::governance::*;
use multiversx_sc::types::MultiValueEncoded;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_set_restricted_vote_nonces_when_contract_calls_itself() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            let mut restricted_nonces = MultiValueEncoded::new();
            restricted_nonces.push(1);
            restricted_nonces.push(3);
            sc.set_restricted_vote_nonces_endpoint(restricted_nonces);

            assert!(sc.restricted_vote_nonces().contains(&1));
            assert!(!sc.restricted_vote_nonces().contains(&2));
            assert!(sc.restricted_vote_nonces().contains(&3));
        })
        .assert_ok();
}

#[test]
fn it_fails_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut restricted_nonces = MultiValueEncoded::new();
            restricted_nonces.push(1);
            restricted_nonces.push(3);
            sc.set_restricted_vote_nonces_endpoint(restricted_nonces);
        })
        .assert_user_error("action not allowed by user");
}
