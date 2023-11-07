use entity::config::*;
use entity::governance::*;
use entity::permission::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_removes_the_governance_token() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.remove_gov_token_endpoint();

            assert!(sc.gov_token_id().is_empty());
            assert!(sc.lock_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID)).is_empty());
        })
        .assert_ok();
}

#[test]
fn it_fails_when_entity_is_leaderless() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup.configure_gov_token(true);
    setup.configure_leaderless();

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.remove_gov_token_endpoint();
        })
        .assert_user_error("not allowed when leaderless");
}

#[test]
fn it_fails_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.remove_gov_token_endpoint();
        })
        .assert_user_error("action not allowed by user");
}
