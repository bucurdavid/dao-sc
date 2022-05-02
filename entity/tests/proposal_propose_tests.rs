use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::vote::*;
use entity::governance::*;
use setup::*;

mod setup;

#[test]
fn it_creates_a_proposal() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let owner_address = setup.owner_address.clone();

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.owner_address,
            &setup.contract,
            ENTITY_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
            |sc| {
                sc.propose_endpoint(
                    managed_buffer!(b"my title"),
                    managed_buffer!(b"my description"),
                    MultiValueManagedVec::from(Vec::<Action<DebugApi>>::new()),
                );
            },
        )
        .assert_ok();

    // assert contract storage
    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let proposal = sc.proposals(0).get();

            assert_eq!(0, proposal.id);
            assert_eq!(managed_address!(&owner_address), proposal.proposer);
            assert_eq!(managed_buffer!(b"my title"), proposal.title);
            assert_eq!(managed_buffer!(b"my description"), proposal.description);
            assert_eq!(false, proposal.was_executed);
            assert_eq!(0, proposal.actions.len());
            assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL), proposal.votes_for);
            assert_eq!(managed_biguint!(0), proposal.votes_against);

            assert_eq!(1, sc.next_proposal_id().get());
        })
        .assert_ok();
}

#[test]
fn it_creates_a_proposal_with_actions() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let user_address = setup.user_address.clone();

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.owner_address,
            &setup.contract,
            ENTITY_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
            |sc| {
                let mut actions = Vec::<Action<DebugApi>>::new();

                actions.push(Action::<DebugApi> {
                    address: managed_address!(&user_address),
                    endpoint: managed_buffer!(b"func"),
                    arguments: ManagedVec::from(vec![managed_buffer!(b"arg1")]),
                    gas_limit: 5_000_000u64,
                    token_id: managed_token_id!(b"EGLD"),
                    token_nonce: 0,
                    amount: managed_biguint!(5),
                });

                actions.push(Action::<DebugApi> {
                    address: managed_address!(&user_address),
                    endpoint: managed_buffer!(b"func"),
                    arguments: ManagedVec::from(vec![managed_buffer!(b"arg1")]),
                    gas_limit: 5_000_000u64,
                    token_id: managed_token_id!(b"EGLD"),
                    token_nonce: 0,
                    amount: managed_biguint!(5),
                });

                sc.propose_endpoint(
                    managed_buffer!(b"my title"),
                    managed_buffer!(b"my description"),
                    MultiValueManagedVec::from(actions),
                );
            },
        )
        .assert_ok();

    // assert contract storage
    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let proposal = sc.proposals(0).get();

            assert_eq!(2, proposal.actions.len());
        })
        .assert_ok();
}

#[test]
fn it_sends_a_vote_nft_to_the_voter() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let owner_address = setup.owner_address.clone();

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.owner_address,
            &setup.contract,
            ENTITY_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
            |sc| {
                sc.propose_endpoint(
                    managed_buffer!(b"my title"),
                    managed_buffer!(b"my description"),
                    MultiValueManagedVec::from(Vec::<Action<DebugApi>>::new()),
                );
            },
        )
        .assert_ok();

    // assert voter received vote nft
    setup.blockchain.execute_in_managed_environment(|| {
        setup.blockchain.check_nft_balance(
            &setup.owner_address,
            VOTE_NFT_TOKEN_ID,
            1,
            &rust_biguint!(1),
            Some(&VoteNFTAttributes::<DebugApi> {
                proposal_id: 0,
                vote_type: VoteType::For,
                vote_weight: managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
                voter: managed_address!(&owner_address),
                payment: EsdtTokenPayment::new(managed_token_id!(ENTITY_TOKEN_ID), 0, managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL)),
            }),
        );
    });
}

#[test]
fn it_fails_if_bad_token() {
    let mut setup = setup::setup_entity(entity::contract_obj);

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.user_address,
            &setup.contract,
            ENTITY_FAKE_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
            |sc| {
                sc.propose_endpoint(
                    managed_buffer!(b""),
                    managed_buffer!(b""),
                    MultiValueManagedVec::from(Vec::<Action<DebugApi>>::new()),
                );
            },
        )
        .assert_user_error("invalid token");
}

#[test]
fn it_fails_if_bad_amount() {
    let mut setup = setup::setup_entity(entity::contract_obj);

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.user_address,
            &setup.contract,
            ENTITY_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL - 1),
            |sc| {
                sc.propose_endpoint(
                    managed_buffer!(b""),
                    managed_buffer!(b""),
                    MultiValueManagedVec::from(Vec::<Action<DebugApi>>::new()),
                );
            },
        )
        .assert_user_error("insufficient vote weight");
}
