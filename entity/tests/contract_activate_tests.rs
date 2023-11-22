use entity::contract::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_fails_activate_contract_when_no_code_staged() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let contract_address = setup.contract.address_ref();

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.activate_contract_endpoint(
                managed_buffer!(b"uniqueid1"),
                managed_address!(contract_address),
                CodeMetadata::DEFAULT,
                MultiValueEncoded::new(),
            );
        })
        .assert_user_error("contract not staged");
}

#[test]
fn it_fails_activate_contract_when_unique_id_not_unique() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let contract_address = setup.contract.address_ref();

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.stage(&managed_address!(contract_address)).set(managed_buffer!(b"dummy_code"));
            sc.stage_lock(&managed_address!(contract_address)).set(true);

            sc.unique_ids().insert(managed_buffer!(b"uniqueid1"));

            sc.activate_contract_endpoint(
                managed_buffer!(b"uniqueid1"),
                managed_address!(contract_address),
                CodeMetadata::DEFAULT,
                MultiValueEncoded::new(),
            );
        })
        .assert_user_error("unique id already exists");
}

#[test]
fn it_fails_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let contract_address = setup.contract.address_ref();

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.activate_contract_endpoint(
                managed_buffer!(b"uniqueid"),
                managed_address!(contract_address),
                CodeMetadata::DEFAULT,
                MultiValueEncoded::new(),
            );
        })
        .assert_user_error("action not allowed by user");
}
