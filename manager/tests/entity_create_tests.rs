use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use manager::config::*;
use manager::*;
use setup::*;
mod setup;

#[test]
#[ignore]
fn it_creates_an_entity() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let caller = setup.blockchain.create_user_account(&rust_biguint!(1));

    setup.blockchain.set_esdt_balance(&caller, COST_TOKEN_ID, &rust_biguint!(5000));

    let manager_addr = setup.contract.address_ref().clone();
    let new_entity_wrapper = setup.blockchain.prepare_deploy_from_sc(&manager_addr, entity::contract_obj);

    setup
        .blockchain
        .execute_esdt_transfer(&caller, &setup.contract, COST_TOKEN_ID, 0, &rust_biguint!(1000), |sc| {
            sc.create_entity_endpoint(MultiValueManagedVec::new());

            let new_entity_address = managed_address!(new_entity_wrapper.address_ref());

            assert!(sc.entities().contains(&new_entity_address));
        })
        .assert_ok();
}

#[test]
fn it_fails_if_wrong_cost_token() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let caller = setup.blockchain.create_user_account(&rust_biguint!(1));
    let wrong_cost_token: &[u8] = b"WRONG-abcdef";

    setup.blockchain.set_esdt_balance(&caller, wrong_cost_token, &rust_biguint!(5000));

    setup
        .blockchain
        .execute_esdt_transfer(&caller, &setup.contract, wrong_cost_token, 0, &rust_biguint!(1000), |sc| {
            sc.create_entity_endpoint(MultiValueManagedVec::new());
        })
        .assert_user_error("invalid cost token");
}

#[test]
fn it_fails_if_wrong_cost_amount() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let caller = setup.blockchain.create_user_account(&rust_biguint!(1));
    let wrong_cost_amount = COST_AMOUNT_ENTITY_CREATION - 1u64;

    setup.blockchain.set_esdt_balance(&caller, COST_TOKEN_ID, &rust_biguint!(5000));

    setup
        .blockchain
        .execute_esdt_transfer(&caller, &setup.contract, COST_TOKEN_ID, 0, &rust_biguint!(wrong_cost_amount), |sc| {
            sc.create_entity_endpoint(MultiValueManagedVec::new());
        })
        .assert_user_error("invalid cost amount");
}
