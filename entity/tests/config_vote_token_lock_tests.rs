use multiversx_sc_scenario::*;
use entity::config::*;
use entity::governance::*;
use entity::Entity;
use setup::*;

mod setup;

#[test]
fn it_changes_the_vote_token_lock_when_called_by_trusted_host() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup.configure_trusted_host();

    setup
        .blockchain
        .execute_tx(&setup.trusted_host_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.change_vote_token_lock_endpoint(managed_token_id!(ENTITY_GOV_TOKEN_ID), true);

            assert!(sc.lock_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID)).get());
        })
        .assert_ok();
}

#[test]
fn it_fails_if_caller_not_trusted_host() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup.configure_trusted_host();

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.change_vote_token_lock_endpoint(managed_token_id!(ENTITY_GOV_TOKEN_ID), true);
        })
        .assert_user_error("action not allowed by user");
}
