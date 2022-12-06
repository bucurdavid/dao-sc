use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::*;
use entity::permission::*;
use setup::*;

mod setup;

#[test]
fn it_sets_the_governance_token_initially_by_leader() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.init_gov_token_endpoint(managed_token_id!(b"GOV-123456"), managed_biguint!(1_000), true);

            assert_eq!(sc.gov_token_id().get(), managed_token_id!(b"GOV-123456"));
        })
        .assert_ok();
}

#[test]
fn it_fails_if_caller_not_leader() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(&setup.user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.init_gov_token_endpoint(managed_token_id!(b"GOV-123456"), managed_biguint!(1_000), true);
        })
        .assert_user_error("caller must be leader");
}

#[test]
fn it_fails_if_gov_token_already_set() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(&setup.user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.gov_token_id().set(managed_token_id!(b"GOV-123456"));

            sc.init_gov_token_endpoint(managed_token_id!(b"NEW-123456"), managed_biguint!(1_000), true);
        })
        .assert_user_error("gov token is already set");
}
