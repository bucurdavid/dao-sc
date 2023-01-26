use multiversx_sc_scenario::*;
use entity::config::*;
use entity::governance::*;
use setup::*;

mod setup;

#[test]
fn it_changes_the_governance_token_if_contract_calls_itself() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.change_gov_token_endpoint(managed_token_id!(b"GOV-123456"), managed_biguint!(1_000), true);

            assert_eq!(sc.gov_token_id().get(), managed_token_id!(b"GOV-123456"));
        })
        .assert_ok();
}

#[test]
fn it_changes_the_governance_token_even_if_supply_lower_than_one_hundred() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.change_gov_token_endpoint(managed_token_id!(b"GOV-123456"), managed_biguint!(5), true);

            assert_eq!(sc.gov_token_id().get(), managed_token_id!(b"GOV-123456"));
        })
        .assert_ok();
}

#[test]
fn it_fails_if_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.change_gov_token_endpoint(managed_token_id!(b"GOV-123456"), managed_biguint!(1_000), true);
        })
        .assert_user_error("action not allowed by user");
}
