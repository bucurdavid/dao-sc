use elrond_wasm_debug::*;
use manager::config::*;
use manager::credits::*;
use manager::features::*;
use setup::*;

mod setup;

#[test]
fn it_recalculates_to_daily_base_cost_if_no_features_set() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let entity_token = b"ENTITY-123456";
    let entity_address = setup.contract_entity_template.address_ref();

    setup.blockchain.set_esdt_balance(&entity_address, COST_TOKEN_ID, &rust_biguint!(100));

    setup
        .blockchain
        .execute_esdt_transfer(&entity_address, &setup.contract, COST_TOKEN_ID, 0, &rust_biguint!(50), |sc| {
            sc.entities_map()
                .insert(managed_token_id!(entity_token), managed_address!(entity_address));

            sc.cost_base_daily_amount().set(managed_biguint!(20));

            sc.recalculate_daily_cost(&managed_token_id!(entity_token));

            let actual = sc.credit_entries(&managed_token_id!(entity_token)).get();

            assert_eq!(managed_biguint!(20), actual.daily_cost);
        })
        .assert_ok();
}

#[test]
fn it_recalculates_daily_cost_with_features() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let entity_token = b"ENTITY-123456";
    let entity_address = setup.contract_entity_template.address_ref();

    setup.blockchain.set_esdt_balance(&entity_address, COST_TOKEN_ID, &rust_biguint!(100));

    setup
        .blockchain
        .execute_esdt_transfer(&entity_address, &setup.contract, COST_TOKEN_ID, 0, &rust_biguint!(50), |sc| {
            sc.entities_map()
                .insert(managed_token_id!(entity_token), managed_address!(entity_address));

            sc.cost_base_daily_amount().set(managed_biguint!(20));

            sc.cost_feature_daily_amount(&managed_buffer!(b"feature1")).set(managed_biguint!(5));
            sc.cost_feature_daily_amount(&managed_buffer!(b"feature2")).set(managed_biguint!(10));

            sc.enable_feature(&managed_token_id!(entity_token), managed_buffer!(b"feature1"));
            sc.enable_feature(&managed_token_id!(entity_token), managed_buffer!(b"feature2"));

            sc.recalculate_daily_cost(&managed_token_id!(entity_token));

            let actual = sc.credit_entries(&managed_token_id!(entity_token)).get();

            assert_eq!(managed_biguint!(35), actual.daily_cost);
        })
        .assert_ok();
}