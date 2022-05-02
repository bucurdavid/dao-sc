use elrond_wasm_debug::*;
use entity::config::*;
use entity::*;
use setup::*;

mod setup;

#[test]
fn it_seals_the_entity() {
    let mut setup = setup::setup_entity(entity::contract_obj);

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(1), |sc| {
            sc.sealed().set(SEALED_NOT_SET);

            sc.seal_endpoint();

            assert_eq!(sc.sealed().get(), SEALED_ON);
        })
        .assert_ok();
}
