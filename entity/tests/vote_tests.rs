use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::vote::*;
use entity::governance::*;
use entity::*;
use setup::*;

mod setup;

#[test]
fn it_votes_for_a_proposal() {
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
                votes_for: managed_biguint!(0),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(0).set(dummy_proposal);
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.vote_for_endpoint(0);

            let proposal = sc.proposals(0).get();

            assert_eq!(managed_biguint!(25), proposal.votes_for);
            assert_eq!(managed_biguint!(0), proposal.votes_against);
            assert_eq!(managed_biguint!(25), sc.protected_vote_tokens().get());
        })
        .assert_ok();

    // same vote again to assert it adds up
    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.vote_for_endpoint(0);

            let proposal = sc.proposals(0).get();

            assert_eq!(managed_biguint!(50), proposal.votes_for);
            assert_eq!(managed_biguint!(0), proposal.votes_against);
            assert_eq!(managed_biguint!(50), sc.protected_vote_tokens().get());
        })
        .assert_ok();
}

#[test]
fn it_votes_against_a_proposal() {
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
                votes_for: managed_biguint!(0),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(0).set(dummy_proposal);
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.vote_against_endpoint(0);

            let proposal = sc.proposals(0).get();

            assert_eq!(managed_biguint!(0), proposal.votes_for);
            assert_eq!(managed_biguint!(25), proposal.votes_against);
            assert_eq!(managed_biguint!(25), sc.protected_vote_tokens().get());
        })
        .assert_ok();

    // same vote again to assert it adds up
    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.vote_against_endpoint(0);

            let proposal = sc.proposals(0).get();

            assert_eq!(managed_biguint!(0), proposal.votes_for);
            assert_eq!(managed_biguint!(50), proposal.votes_against);
            assert_eq!(managed_biguint!(50), sc.protected_vote_tokens().get());
        })
        .assert_ok();
}

#[test]
fn it_sends_a_vote_nft_to_the_voter() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let user_address = setup.user_address.clone();
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.owner_address,
            &setup.contract,
            ENTITY_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
            |sc| {
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
                    votes_for: managed_biguint!(5),
                    proposer: managed_address!(&Address::zero()),
                    was_executed: false,
                };

                sc.proposals(0).set(dummy_proposal);
            },
        )
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&setup.user_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.vote_for_endpoint(0);
        })
        .assert_ok();

    // assert voter received vote nft
    setup.blockchain.execute_in_managed_environment(|| {
        setup.blockchain.check_nft_balance(
            &setup.user_address,
            VOTE_NFT_TOKEN_ID,
            1,
            &rust_biguint!(1),
            Some(&VoteNFTAttributes::<DebugApi> {
                proposal_id: 0,
                vote_type: VoteType::For,
                vote_weight: managed_biguint!(25),
                voter: managed_address!(&user_address),
                payment: EsdtTokenPayment::new(managed_token_id!(ENTITY_TOKEN_ID), 0, managed_biguint!(25)),
            }),
        );
    });
}

#[test]
fn it_fails_if_proposal_voting_period_has_ended() {
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
                votes_for: managed_biguint!(5),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(0).set(dummy_proposal);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.vote_against_endpoint(0);
        })
        .assert_user_error("proposal is not active");
}

#[test]
fn it_fails_if_proposal_is_pending() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            let starts_at = 1u64;
            let ends_at = starts_at + voting_period_seconds;

            let dummy_proposal = Proposal::<DebugApi> {
                actions: ManagedVec::from(Vec::<Action<DebugApi>>::new()),
                starts_at,
                ends_at,
                title: managed_buffer!(b""),
                description: managed_buffer!(b""),
                id: 0,
                votes_against: managed_biguint!(0),
                votes_for: managed_biguint!(5),
                proposer: managed_address!(&Address::zero()),
                was_executed: false,
            };

            sc.proposals(0).set(dummy_proposal);
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.vote_against_endpoint(0);
        })
        .assert_user_error("proposal is not active");
}
