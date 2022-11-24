use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use manager::config::*;
use manager::features::*;
use manager::*;

mod setup;

#[test]
fn it_sets_features() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let feature = b"myfeature";
    let entity_address = setup.contract_entity_template.address_ref();

    setup
        .blockchain
        .execute_tx(&entity_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut args = MultiValueManagedVec::new();
            args.push(managed_buffer!(feature));

            sc.entities().insert(managed_address!(&entity_address));

            sc.set_features_endpoint(args);

            assert!(sc.features(&managed_address!(&entity_address)).contains(&managed_buffer!(feature)));
        })
        .assert_ok();
}

#[test]
fn it_fails_if_the_entity_does_not_exist() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let feature = b"myfeature";

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut args = MultiValueManagedVec::new();
            args.push(managed_buffer!(feature));

            sc.set_features_endpoint(args);
        })
        .assert_user_error("entity does not exist");
}
