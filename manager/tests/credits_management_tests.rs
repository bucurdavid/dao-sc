use manager::config::*;
use manager::credits::*;
use multiversx_sc_scenario::*;

mod setup;

#[test]
fn it_sets_the_bonus_factor() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let admin = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.admins().insert(managed_address!(&admin));

            sc.set_credits_bonus_factor_endpoint(2);

            assert_eq!(2, sc.credits_bonus_factor().get());
        })
        .assert_ok();
}

#[test]
fn it_fails_to_set_bonus_factor_when_not_admin() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let user = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup
        .blockchain
        .execute_tx(&user, &setup.contract, &rust_biguint!(0), |sc| {
            sc.set_credits_bonus_factor_endpoint(2);
        })
        .assert_user_error("caller must be admin");
}
