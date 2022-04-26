use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use manager::*;
use setup::*;
mod setup;

#[test]
#[ignore]
fn it_creates_an_entity() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let caller = setup.blockchain.create_user_account(&rust_biguint!(1));
    let token_name = b"Token";
    let token_id = b"Token-123456";
    let token_supply = 100_000u64;

    setup.blockchain.set_esdt_balance(&caller, COST_TOKEN_ID, &rust_biguint!(5000));

    setup
        .blockchain
        .execute_esdt_transfer(&caller, &setup.contract, COST_TOKEN_ID, 0, &rust_biguint!(1000), |sc| {
            sc.setup_token_id(&managed_address!(&caller)).set(&managed_token_id!(token_id));
            sc.setup_token_supply(&managed_address!(&caller)).set(&managed_biguint!(token_supply));

            sc.create_entity_endpoint(managed_token_id!(token_id), MultiValueEncoded::new());

            assert!(sc.entities_map().get(&managed_token_id!(token_id)).is_some());
        })
        .assert_ok();
}
