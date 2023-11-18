use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_cancels_a_proposal() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_esdt_transfer(
            &proposer_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(ENTITY_GOV_TOKEN_SUPPLY),
            |sc| {
                proposal_id = sc.propose_endpoint(
                    managed_buffer!(b"host_id"),
                    managed_buffer!(b"content_hash"),
                    managed_buffer!(b"content_sig"),
                    managed_buffer!(b""),
                    POLL_DEFAULT_ID,
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 - 1);

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.cancel_proposal_endpoint(proposal_id);

            let actual_proposal = sc.proposals(proposal_id).get();
            let actual_status = sc.get_proposal_status(&actual_proposal);

            assert_eq!(actual_status, ProposalStatus::Canceled);
        })
        .assert_ok();
}

#[test]
fn it_fails_when_caller_not_proposer() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let user_address = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_esdt_transfer(
            &proposer_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(ENTITY_GOV_TOKEN_SUPPLY),
            |sc| {
                proposal_id = sc.propose_endpoint(
                    managed_buffer!(b"host_id"),
                    managed_buffer!(b"content_hash"),
                    managed_buffer!(b"content_sig"),
                    managed_buffer!(b""),
                    POLL_DEFAULT_ID,
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 - 1);

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.cancel_proposal_endpoint(proposal_id);
        })
        .assert_user_error("proposer must cancel proposal");
}

#[test]
fn it_fails_when_proposal_not_active() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_esdt_transfer(
            &proposer_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(ENTITY_GOV_TOKEN_SUPPLY),
            |sc| {
                proposal_id = sc.propose_endpoint(
                    managed_buffer!(b"host_id"),
                    managed_buffer!(b"content_hash"),
                    managed_buffer!(b"content_sig"),
                    managed_buffer!(b""),
                    POLL_DEFAULT_ID,
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.cancel_proposal_endpoint(proposal_id);
        })
        .assert_user_error("proposal is not active");
}
