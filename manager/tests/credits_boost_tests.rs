use elrond_wasm_debug::*;
use manager::config::*;
use manager::credits::*;
use setup::*;

mod setup;

#[test]
fn it_increases_deposited_amounts_in_the_storage() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let entity_token = b"ENTITY-123456";
    let entity_address = setup.contract_entity_template.address_ref();

    setup.blockchain.set_esdt_balance(&entity_address, COST_TOKEN_ID, &rust_biguint!(100));

    setup
        .blockchain
        .execute_esdt_transfer(&entity_address, &setup.contract, COST_TOKEN_ID, 0, &rust_biguint!(50), |sc| {
            sc.entities_map()
                .insert(managed_token_id!(entity_token), managed_address!(entity_address));

            sc.boost_endpoint(managed_token_id!(entity_token));

            let actual = sc.credit_entries(&managed_address!(entity_address)).get();

            assert_eq!(managed_biguint!(50), actual.total_amount);
            assert_eq!(managed_biguint!(50), actual.period_amount);
            assert_eq!(managed_biguint!(50), sc.credit_total_deposits_amount().get());
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&entity_address, &setup.contract, COST_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.boost_endpoint(managed_token_id!(entity_token));

            let actual = sc.credit_entries(&managed_address!(entity_address)).get();

            assert_eq!(managed_biguint!(75), actual.total_amount);
            assert_eq!(managed_biguint!(75), actual.period_amount);
            assert_eq!(managed_biguint!(75), sc.credit_total_deposits_amount().get());
        })
        .assert_ok();
}

#[test]
fn it_fails_if_the_entity_does_not_exist() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let entity_token = b"ENTITY-123456";
    let entity_address = setup.contract_entity_template.address_ref();

    setup.blockchain.set_esdt_balance(&entity_address, COST_TOKEN_ID, &rust_biguint!(100));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.boost_endpoint(managed_token_id!(entity_token));
        })
        .assert_user_error("entity does not exist");
}
