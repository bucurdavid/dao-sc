use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use entity::governance::*;
use setup::*;

mod setup;

#[test]
fn it_mints_if_contract_calls_itself() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .set_esdt_local_roles(setup.contract.address_ref(), b"TOKEN-123456", &[EsdtLocalRole::Mint]);

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.mint_endpoint(managed_token_id!(b"TOKEN-123456"), 0, managed_biguint!(1_000));
        })
        .assert_ok();

    // TODO: add balance check
}

#[test]
fn it_fails_if_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.mint_endpoint(managed_token_id!(b"TOKEN-123456"), 0, managed_biguint!(1_000));
        })
        .assert_user_error("action not allowed by user");
}
