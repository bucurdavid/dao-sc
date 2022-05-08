use elrond_wasm::elrond_codec::multi_types::*;
use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use manager::config::*;
use manager::features::*;

mod setup;

#[test]
fn it_sets_features() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let token_id = b"Token-123456";
    let feature = b"myfeature";
    let entity_address = setup.contract_entity_template.address_ref();

    setup
        .blockchain
        .execute_tx(&entity_address, &setup.contract, &rust_biguint!(0), |sc| {
            let entity_token_id = managed_token_id!(token_id);

            let mut args = MultiValueEncoded::new();
            args.push(MultiValue2::from((managed_buffer!(feature), ManagedBuffer::from(b"true"))));

            sc.entities_map().insert(entity_token_id.clone(), managed_address!(entity_address));

            sc.set_features_endpoint(entity_token_id.clone(), args);

            assert!(sc.features(&entity_token_id).contains(&managed_buffer!(feature)));
        })
        .assert_ok();
}

#[test]
fn it_fails_if_the_entity_does_not_exist() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let token_id = b"Token-123456";
    let feature = b"myfeature";

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut args = MultiValueEncoded::new();
            args.push(MultiValue2::from((managed_buffer!(feature), ManagedBuffer::from(b"true"))));

            sc.set_features_endpoint(managed_token_id!(token_id), args);
        })
        .assert_user_error("entity does not exist");
}
