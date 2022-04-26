use elrond_wasm_debug::*;
use manager::*;

mod setup;

#[test]
fn it_registers_an_entity_token() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let caller = setup.blockchain.create_user_account(&rust_biguint!(1));
    let entity_token_id = b"ENTITY-123456";

    setup.blockchain.set_esdt_balance(&caller, entity_token_id, &rust_biguint!(5000));

    setup
        .blockchain
        .execute_esdt_transfer(&caller, &setup.contract, entity_token_id, 0, &rust_biguint!(1), |sc| {
            sc.register_entity_token_endpoint(managed_biguint!(100_000));

            assert_eq!(sc.setup_token_id(&managed_address!(&caller)).get(), managed_token_id!(entity_token_id));
            assert_eq!(sc.setup_token_supply(&managed_address!(&caller)).get(), managed_biguint!(100_000));
        })
        .assert_ok();
}
