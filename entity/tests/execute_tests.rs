use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::*;

mod setup;

#[test]
#[ignore]
fn it_executes_actions_of_a_proposal() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let starts_at = 0u64;
            let voting_period_minutes = sc.voting_period_in_minutes().get() as u64;
            let ends_at = starts_at + voting_period_minutes * 60;

            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                address: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b""),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                token_id: managed_token_id!(b"EGLD"),
                token_nonce: 0,
                amount: managed_biguint!(5),
            });

            let dummy_proposal = Proposal::<DebugApi> {
                actions: ManagedVec::from(Vec::<Action<DebugApi>>::new()),
                starts_at,
                ends_at,
                title: managed_buffer!(b""),
                description: managed_buffer!(b""),
                id: 0,
                votes_against: managed_biguint!(0),
                votes_for: sc.quorum().get(),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(0).set(dummy_proposal);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.execute_endpoint(0);
        })
        .assert_ok();

    setup.blockchain.check_egld_balance(&action_receiver, &rust_biguint!(5));
}

#[test]
fn it_marks_a_proposal_as_executed() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let starts_at = 0u64;
            let voting_period_minutes = sc.voting_period_in_minutes().get() as u64;
            let ends_at = starts_at + voting_period_minutes * 60;

            let dummy_proposal = Proposal::<DebugApi> {
                actions: ManagedVec::from(Vec::<Action<DebugApi>>::new()),
                starts_at,
                ends_at,
                title: managed_buffer!(b""),
                description: managed_buffer!(b""),
                id: 0,
                votes_against: managed_biguint!(0),
                votes_for: sc.quorum().get(),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(0).set(dummy_proposal);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.execute_endpoint(0);

            let proposal = sc.proposals(0).get();
            assert_eq!(true, proposal.was_executed);
        })
        .assert_ok();
}

#[test]
fn it_fails_if_attempted_to_execute_again() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let starts_at = 0u64;
            let voting_period_minutes = sc.voting_period_in_minutes().get() as u64;
            let ends_at = starts_at + voting_period_minutes * 60;

            let dummy_proposal = Proposal::<DebugApi> {
                actions: ManagedVec::from(Vec::<Action<DebugApi>>::new()),
                starts_at,
                ends_at,
                title: managed_buffer!(b""),
                description: managed_buffer!(b""),
                id: 0,
                votes_against: managed_biguint!(0),
                votes_for: sc.quorum().get(),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(0).set(dummy_proposal);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.execute_endpoint(0);

            sc.execute_endpoint(0); // and again
        })
        .assert_user_error("proposal is not executable");
}

#[test]
fn it_fails_if_the_proposal_is_still_active() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let starts_at = 0u64;
            let ends_at = starts_at + voting_period_seconds;

            let dummy_proposal = Proposal::<DebugApi> {
                actions: ManagedVec::from(Vec::<Action<DebugApi>>::new()),
                starts_at,
                ends_at,
                title: managed_buffer!(b""),
                description: managed_buffer!(b""),
                id: 0,
                votes_against: managed_biguint!(0),
                votes_for: sc.quorum().get(),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(0).set(dummy_proposal);

            sc.execute_endpoint(0);
        })
        .assert_user_error("proposal is not executable");
}
