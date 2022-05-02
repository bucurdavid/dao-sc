use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::governance::proposal::*;
use entity::governance::*;
use setup::*;

mod setup;

#[test]
fn it_fails_if_bad_token() {
    let mut setup = setup::setup_entity(entity::contract_obj);

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.user_address,
            &setup.contract,
            ENTITY_FAKE_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
            |sc| {
                sc.propose_endpoint(
                    managed_buffer!(&b""[..]),
                    managed_buffer!(&b""[..]),
                    MultiValueManagedVec::from(Vec::<Action<DebugApi>>::new()),
                );
            },
        )
        .assert_user_error("invalid token");
}

#[test]
fn it_fails_if_bad_amount() {
    let mut setup = setup::setup_entity(entity::contract_obj);

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.user_address,
            &setup.contract,
            ENTITY_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL - 1),
            |sc| {
                sc.propose_endpoint(
                    managed_buffer!(&b""[..]),
                    managed_buffer!(&b""[..]),
                    MultiValueManagedVec::from(Vec::<Action<DebugApi>>::new()),
                );
            },
        )
        .assert_user_error("insufficient vote weight");
}
