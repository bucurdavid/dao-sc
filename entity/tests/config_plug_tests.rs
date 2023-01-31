use entity::config::*;
use entity::governance::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_configures_a_plug() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let plug_address = setup.contract.address_ref();

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.set_plug_endpoint(managed_address!(plug_address), managed_biguint!(1000), managed_biguint!(50));

            assert_eq!(sc.quorum().get(), managed_biguint!(1000));
            assert_eq!(sc.min_propose_weight().get(), managed_biguint!(50));
        })
        .assert_ok();
}

#[test]
fn it_fails_to_set_a_plug_when_governance_token_already_set() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let plug_address = setup.contract.address_ref().clone();

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.set_plug_endpoint(managed_address!(&plug_address), managed_biguint!(2000), managed_biguint!(100));
        })
        .assert_user_error("already has vote token");
}

#[test]
fn it_fails_to_set_a_plug_twice() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let plug_address = setup.contract.address_ref();

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.set_plug_endpoint(managed_address!(plug_address), managed_biguint!(1000), managed_biguint!(50));

            sc.set_plug_endpoint(managed_address!(plug_address), managed_biguint!(2000), managed_biguint!(100));
        })
        .assert_user_error("already plugged");
}

#[test]
fn it_fails_if_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let plug_address = setup.contract.address_ref();

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.set_plug_endpoint(managed_address!(plug_address), managed_biguint!(1000), managed_biguint!(50));
        })
        .assert_user_error("action not allowed by user");
}
