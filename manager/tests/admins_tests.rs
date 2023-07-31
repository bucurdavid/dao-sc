use manager::config::*;
use manager::*;
use multiversx_sc_scenario::*;

mod setup;

#[test]
fn it_adds_an_admin() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let admin = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.add_admin_endpoint(managed_address!(&admin));

            assert!(sc.admins().contains(&managed_address!(&admin)));
        })
        .assert_ok();
}

#[test]
fn it_removes_an_admin() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let admin = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.admins().insert(managed_address!(&admin));

            sc.remove_admin_endpoint(managed_address!(&admin));

            assert!(!sc.admins().contains(&managed_address!(&admin)));
        })
        .assert_ok();
}

#[test]
fn it_fails_when_caller_not_admin() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let user = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup
        .blockchain
        .execute_tx(&user, &setup.contract, &rust_biguint!(0), |sc| {
            sc.add_admin_endpoint(managed_address!(&user));
        })
        .assert_user_error("caller must be admin");

    setup
        .blockchain
        .execute_tx(&user, &setup.contract, &rust_biguint!(0), |sc| {
            sc.remove_admin_endpoint(managed_address!(&user));
        })
        .assert_user_error("caller must be admin");
}
