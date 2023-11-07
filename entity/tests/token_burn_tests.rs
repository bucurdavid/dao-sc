use entity::governance::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_burns_when_contract_calls_itself() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .set_esdt_local_roles(setup.contract.address_ref(), b"TOKEN-123456", &[EsdtLocalRole::Burn]);

    setup
        .blockchain
        .set_esdt_balance(setup.contract.address_ref(), b"TOKEN-123456", &rust_biguint!(1_000));

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.burn_endpoint(managed_token_id!(b"TOKEN-123456"), 0, managed_biguint!(1_000));
        })
        .assert_ok();

    // TODO: add balance check
}

#[test]
fn it_fails_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.burn_endpoint(managed_token_id!(b"TOKEN-123456"), 0, managed_biguint!(1_000));
        })
        .assert_user_error("action not allowed by user");
}
