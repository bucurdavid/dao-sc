use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use entity::permission::*;
use setup::*;

mod setup;

#[test]
fn it_creates_a_permission() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let sc_address = setup.contract.address_ref();

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_permission_endpoint(
                managed_buffer!(b"testperm"),
                managed_biguint!(0),
                managed_address!(sc_address),
                managed_buffer!(b"endpoint"),
                MultiValueManagedVec::new(),
            );

            assert!(sc.permissions().contains(&managed_buffer!(b"testperm")));

            let actual_permission_details = sc.permission_details(&managed_buffer!(b"testperm")).get();

            assert_eq!(managed_address!(sc_address), actual_permission_details.destination);
            assert_eq!(managed_buffer!(b"endpoint"), actual_permission_details.endpoint);
        })
        .assert_ok();
}
