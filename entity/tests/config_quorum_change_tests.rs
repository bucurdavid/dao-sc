use entity::config::*;
use entity::governance::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_changes_the_quorum_when_contract_calls_itself() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.change_quorum_endpoint(managed_biguint!(1000));

            assert_eq!(sc.quorum().get(), managed_biguint!(1000));
        })
        .assert_ok();
}

#[test]
fn it_fails_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.change_quorum_endpoint(managed_biguint!(1000));
        })
        .assert_user_error("action not allowed by user");
}
