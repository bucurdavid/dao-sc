use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::*;
use setup::*;

mod setup;

#[test]
fn it_redeems_vote_nfts() {
    let mut setup = setup::setup_entity(entity::contract_obj);
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
                sc.propose_endpoint(
                    managed_buffer!(b"my title"),
                    managed_buffer!(b"my description"),
                    MultiValueManagedVec::from(Vec::<Action<DebugApi>>::new()),
                );
            },
        )
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, VOTE_NFT_TOKEN_ID, 1, &rust_biguint!(1), |sc| {
            sc.redeem_endpoint();
        })
        .assert_ok();

    let owner_address = setup.owner_address.clone();
    setup
        .blockchain
        .check_esdt_balance(&owner_address, ENTITY_TOKEN_ID, &rust_biguint!(ENTITY_TOKEN_SUPPLY));
}

#[test]
fn it_fails_if_voting_period_has_not_ended() {
    let mut setup = setup::setup_entity(entity::contract_obj);

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

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, VOTE_NFT_TOKEN_ID, 1, &rust_biguint!(1), |sc| {
            sc.redeem_endpoint();
        })
        .assert_user_error("proposal is still active");
}
