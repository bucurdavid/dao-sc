use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::*;

mod setup;

#[test]
fn it_changes_the_governance_token() {
    let mut setup = setup::setup_entity(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.change_gov_token_endpoint(managed_token_id!(b"GOV-123456"));

            assert_eq!(sc.governance_token_id().get(), managed_token_id!(b"GOV-123456"));
        })
        .assert_ok();
}

#[test]
fn it_fails_if_entity_is_sealed_even_if_contract_calls_itself() {
    let mut setup = setup::setup_entity(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.sealed().set(SEALED_ON);

            sc.change_gov_token_endpoint(managed_token_id!(b"GOV-123456"));
        })
        .assert_user_error("entity is sealed");
}

#[test]
fn it_fails_if_the_entity_is_sealed() {
    let mut setup = setup::setup_entity(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.sealed().set(SEALED_ON);

            sc.change_gov_token_endpoint(managed_token_id!(b"GOV-123456"));
        })
        .assert_user_error("entity is sealed");
}
