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
            sc.set_plug_endpoint(managed_address!(plug_address), managed_biguint!(1000), managed_biguint!(50), 0);

            assert_eq!(sc.quorum().get(), managed_biguint!(1000));
            assert_eq!(sc.min_propose_weight().get(), managed_biguint!(50));
        })
        .assert_ok();
}

#[test]
fn it_fails_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let plug_address = setup.contract.address_ref();

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.set_plug_endpoint(managed_address!(plug_address), managed_biguint!(1000), managed_biguint!(50), 0);
        })
        .assert_user_error("action not allowed by user");
}

#[test]
fn it_fails_when_invalid_weight_decimals() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let plug_address = setup.contract.address_ref();
    let invalid_weight_decimals = 19; // max is derived from max token decimals which is 18

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.set_plug_endpoint(managed_address!(plug_address), managed_biguint!(1000), managed_biguint!(50), invalid_weight_decimals);
        })
        .assert_user_error("invalid weight decimals");
}
