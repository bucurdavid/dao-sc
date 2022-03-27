use elrond_wasm::storage::mappers::StorageTokenWrapper;
use elrond_wasm_debug::{managed_token_id, rust_biguint};
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
            sc.vote_nft_token().set_token_id(&managed_token_id!(VOTE_NFT_TOKEN_ID));

            sc.seal_endpoint();

            assert_eq!(sc.sealed().get(), SEALED_ON);
        })
        .assert_ok();
}
