use entity::config::*;
use entity::governance::*;
use multiversx_sc::codec::multi_types::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_withdraws_tokens_for_all_voters() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voter1_address = &setup.blockchain.create_user_account(&rust_biguint!(0));
    let voter2_address = &setup.blockchain.create_user_account(&rust_biguint!(0));
    let voter3_address = &setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0u64;

    setup.configure_gov_token(true);

    setup.blockchain.set_esdt_balance(&voter1_address, ENTITY_GOV_TOKEN_ID, &rust_biguint!(5));
    setup.blockchain.set_esdt_balance(&voter2_address, ENTITY_GOV_TOKEN_ID, &rust_biguint!(10));
    setup.blockchain.set_esdt_balance(&voter3_address, ENTITY_GOV_TOKEN_ID, &rust_biguint!(20));

    setup
        .blockchain
        .execute_esdt_transfer(&voter1_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(5), |sc| {
            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b"content hash"),
                managed_buffer!(b"content signature"),
                managed_buffer!(b""),
                POLL_DEFAULT_ID,
                MultiValueManagedVec::new(),
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&voter2_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(5), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&voter3_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(5), |sc| {
            sc.vote_against_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_tx(&setup.user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.withdraw_all_endpoint(proposal_id);

            let voter1_id = sc.users().get_user_id(&managed_address!(&voter1_address));
            let voter2_id = sc.users().get_user_id(&managed_address!(&voter2_address));
            let voter3_id = sc.users().get_user_id(&managed_address!(&voter3_address));

            assert!(!sc.withdrawable_voters(proposal_id).contains(&voter1_id));
            assert!(!sc.withdrawable_voters(proposal_id).contains(&voter2_id));
            assert!(!sc.withdrawable_voters(proposal_id).contains(&voter3_id));

            assert!(sc.withdrawable_votes(proposal_id, &managed_address!(&voter1_address)).is_empty());
            assert!(sc.withdrawable_votes(proposal_id, &managed_address!(&voter2_address)).is_empty());
            assert!(sc.withdrawable_votes(proposal_id, &managed_address!(&voter3_address)).is_empty());
        })
        .assert_ok();

    // assert that vote tokens are back in the user's wallet
    setup.blockchain.check_esdt_balance(&voter1_address, ENTITY_GOV_TOKEN_ID, &rust_biguint!(5));
    setup.blockchain.check_esdt_balance(&voter2_address, ENTITY_GOV_TOKEN_ID, &rust_biguint!(10));
    setup.blockchain.check_esdt_balance(&voter3_address, ENTITY_GOV_TOKEN_ID, &rust_biguint!(20));
}
