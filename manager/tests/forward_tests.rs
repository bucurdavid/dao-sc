use manager::*;
use multiversx_sc_scenario::*;

mod setup;

#[test]
fn it_forwards_a_token() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let token_id = b"TOKEN-123456";

    setup.blockchain.set_esdt_balance(setup.contract.address_ref(), token_id, &rust_biguint!(1000));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.forward_token_endpoint(managed_token_id!(token_id), managed_biguint!(500), managed_address!(&receiver));
        })
        .assert_ok();

    setup.blockchain.check_esdt_balance(setup.contract.address_ref(), token_id, &rust_biguint!(500))
}
