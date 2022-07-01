use elrond_wasm_debug::*;
use entity::governance::*;
use entity::config::*;
use setup::*;

mod setup;

#[test]
fn it_issues_a_governance_token() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let caller = setup.blockchain.create_user_account(&rust_biguint!(1));

    setup.blockchain.execute_tx(&caller, &setup.contract, &rust_biguint!(0.5), |sc| {
        sc.issue_gov_token_endpoint(managed_buffer!(b"Token"), managed_buffer!(b"Token-123456"), managed_biguint!(100_000));

        assert!(!sc.quorum().is_empty());
        assert!(!sc.min_proposal_vote_weight().is_empty());
        assert_eq!(managed_token_id!(ENTITY_GOV_TOKEN_ID), sc.gov_token_id().get());
    })
    .assert_ok();
}
