use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::permission::*;
use setup::*;

mod setup;

#[test]
fn it_creates_a_role() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"testrole"));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert!(sc.roles().contains(&managed_buffer!(b"testrole")));
        })
        .assert_ok();
}

#[test]
fn it_creates_a_permission() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let sc_address = setup.contract.address_ref();

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_permission(managed_buffer!(b"testperm"), managed_biguint!(0), managed_address!(sc_address), managed_buffer!(b"endpoint"), ManagedVec::new(), ManagedVec::new());
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert!(sc.permissions().contains(&managed_buffer!(b"testperm")));

            let actual_permission_details = sc.permission_details(&managed_buffer!(b"testperm")).get();

            assert_eq!(managed_address!(sc_address), actual_permission_details.destination);
            assert_eq!(managed_buffer!(b"endpoint"), actual_permission_details.endpoint);
        })
        .assert_ok();
}

#[test]
fn it_assigns_a_role() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"testrole"));

            sc.assign_role(managed_address!(user_address), managed_buffer!(b"testrole"));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let user_id = sc.users().get_user_id(&managed_address!(user_address));

            assert!(sc.user_roles(user_id).contains(&managed_buffer!(b"testrole")));
            assert_eq!(1, sc.roles_member_amount(&managed_buffer!(b"testrole")).get());
        })
        .assert_ok();
}

#[test]
fn it_only_increases_role_member_count_once_per_assigned_user() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"testrole"));
            sc.assign_role(managed_address!(user_address), managed_buffer!(b"testrole"));
            sc.assign_role(managed_address!(user_address), managed_buffer!(b"testrole"));
            // same user again
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(1, sc.roles_member_amount(&managed_buffer!(b"testrole")).get());
        })
        .assert_ok();
}
