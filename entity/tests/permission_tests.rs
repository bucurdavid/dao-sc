use elrond_wasm_debug::*;
use entity::config::*;
use entity::permission::PermissionModule;
use setup::*;

mod setup;

#[test]
fn it_assigns_a_role() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.owner_address;

    setup.blockchain.execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.create_role_endpoint(managed_buffer!(b"testrole"));

        sc.assign_role_endpoint(managed_address!(user_address), managed_buffer!(b"testrole"));
    }).assert_ok();

    setup.blockchain.execute_query(&setup.contract, |sc| {
        let user_id = sc.users().get_user_id(&managed_address!(user_address));

        assert!(sc.user_roles(user_id).contains(&managed_buffer!(b"testrole")));
        assert_eq!(1, sc.roles_member_amount(&managed_buffer!(b"testrole")).get())
    }).assert_ok();
}
