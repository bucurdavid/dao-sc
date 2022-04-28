use elrond_wasm_debug::*;
use manager::*;
use setup::*;
mod setup;

#[test]
fn it_finalizes_an_entity() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let caller = setup.blockchain.create_user_account(&rust_biguint!(1));
    let token_id = b"Token-123456";
    let token_supply = 100_000u64;
    let entity_contract_address = setup.contract_entity_template.address_ref();

    setup.blockchain.set_esdt_balance(&caller, COST_TOKEN_ID, &rust_biguint!(5000));

    setup
        .blockchain
        .execute_tx(&caller, &setup.contract, &rust_biguint!(0), |sc| {
            sc.setup_token_id(&managed_address!(&caller)).set(&managed_token_id!(token_id));
            sc.setup_token_supply(&managed_address!(&caller)).set(&managed_biguint!(token_supply));

            sc.entities_map()
                .insert(managed_token_id!(token_id), managed_address!(entity_contract_address));

            sc.finalize_entity_endpoint(managed_token_id!(token_id));

            assert!(sc.setup_token_id(&managed_address!(&caller)).is_empty());
            assert!(sc.setup_token_supply(&managed_address!(&caller)).is_empty());
        })
        .assert_ok();
}

#[test]
fn it_fails_if_token_is_not_in_setup_mode() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let caller = setup.blockchain.create_user_account(&rust_biguint!(1));

    setup.blockchain.set_esdt_balance(&caller, COST_TOKEN_ID, &rust_biguint!(5000));

    setup
        .blockchain
        .execute_tx(&caller, &setup.contract, &rust_biguint!(0), |sc| {
            sc.finalize_entity_endpoint(managed_token_id!(b"Token-123456"));
        })
        .assert_user_error("token not in setup");
}
