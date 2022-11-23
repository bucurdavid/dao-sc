use elrond_wasm::types::MultiValueManagedVec;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::*;
use setup::*;

mod setup;

#[test]
fn it_fails_to_verify_trusted_host_when_no_signature_given() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let original_trusted_host = setup.trusted_host_address;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            // configure trusted host
            sc.trusted_host_address().set(managed_address!(&original_trusted_host));

            let _ = sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b""),
                managed_buffer!(b""),
                managed_buffer!(b""),
                POLL_DEFAULT_ID,
                MultiValueManagedVec::new(),
            );
        })
        .assert_user_error("not a trusted host");
}
