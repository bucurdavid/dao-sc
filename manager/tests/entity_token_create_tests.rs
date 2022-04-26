use elrond_wasm_debug::*;
use manager::*;

mod setup;

#[test]
fn it_creates_an_entity_token() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let caller = setup.blockchain.create_user_account(&rust_biguint!(1));

    setup
        .blockchain
        .execute_tx(&caller, &setup.contract, &rust_biguint!(0.5), |sc| {
            sc.create_entity_token_endpoint(managed_buffer!(b"Token"), managed_buffer!(b"Token-123456"), managed_biguint!(100_000));

            assert_eq!(sc.setup_token_id(&managed_address!(&caller)).get(), managed_token_id!(b"Token-123456"));
            assert_eq!(sc.setup_token_supply(&managed_address!(&caller)).get(), managed_biguint!(100_000));
        })
        .assert_ok();
}
