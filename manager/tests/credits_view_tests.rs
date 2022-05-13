use elrond_wasm_debug::*;
use manager::config::*;
use manager::credits::*;

mod setup;

#[test]
fn it_returns_available_credits() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let entity_token = b"ENTITY-123456";
    let entity_address = setup.contract_entity_template.address_ref();

    // prepare
    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.entities_map()
                .insert(managed_token_id!(entity_token), managed_address!(entity_address));

            let expand_val = managed_biguint!(1_000000000000000000u64);
            let entry = CreditEntry {
                total_amount: &managed_biguint!(250) * &expand_val,
                period_amount: &managed_biguint!(100) * &expand_val,
                daily_cost: &managed_biguint!(20) * &expand_val,
                last_period_change: 0u64,
            };

            sc.credit_entries(&managed_token_id!(entity_token)).set(entry);
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let expand_val = managed_biguint!(1_000000000000000000u64);

            let (available, daily_cost) = sc.get_credits_view(managed_token_id!(entity_token)).into_tuple();

            assert!(available > managed_biguint!(99) * &expand_val && available < managed_biguint!(101) * &expand_val);
            assert_eq!(daily_cost, &managed_biguint!(20) * &expand_val)
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(60 * 60 * 24 * 1);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let expand_val = managed_biguint!(1_000000000000000000u64);

            let (available, daily_cost) = sc.get_credits_view(managed_token_id!(entity_token)).into_tuple();

            assert!(available > managed_biguint!(79) * &expand_val && available < managed_biguint!(81) * &expand_val);
            assert_eq!(daily_cost, &managed_biguint!(20) * &expand_val)
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(60 * 60 * 24 * 2);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let expand_val = managed_biguint!(1_000000000000000000u64);

            let (available, daily_cost) = sc.get_credits_view(managed_token_id!(entity_token)).into_tuple();

            assert!(available > managed_biguint!(59) * &expand_val && available < managed_biguint!(61) * &expand_val);
            assert_eq!(daily_cost, &managed_biguint!(20) * &expand_val)
        })
        .assert_ok();
}
