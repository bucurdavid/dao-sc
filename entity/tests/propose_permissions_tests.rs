use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::*;
use entity::permission::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_sets_the_longest_policy_voting_period_for_the_proposal() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let sc_address = setup.contract.address_ref().clone();
    let proposer_address = setup.user_address.clone();
    let longest_voting_period_minutes: usize = 180;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"testrole"));

            sc.create_permission(
                managed_buffer!(b"testperm1"),
                managed_biguint!(0),
                managed_address!(&sc_address),
                managed_buffer!(b"testendpoint"),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_permission(
                managed_buffer!(b"testperm2"),
                managed_biguint!(0),
                managed_address!(&sc_address),
                managed_buffer!(b"testendpoint"),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_permission(
                managed_buffer!(b"testperm3"),
                managed_biguint!(0),
                managed_address!(&sc_address),
                managed_buffer!(b"testendpoint"),
                ManagedVec::new(),
                ManagedVec::new(),
            );

            sc.create_policy(
                managed_buffer!(b"testrole"),
                managed_buffer!(b"testperm1"),
                PolicyMethod::Weight,
                managed_biguint!(2u64),
                60,
            );
            sc.create_policy(
                managed_buffer!(b"testrole"),
                managed_buffer!(b"testperm2"),
                PolicyMethod::Weight,
                managed_biguint!(5u64),
                longest_voting_period_minutes,
            );
            sc.create_policy(
                managed_buffer!(b"testrole"),
                managed_buffer!(b"testperm3"),
                PolicyMethod::Weight,
                managed_biguint!(8u64),
                120,
            );

            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"testrole"));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(MIN_PROPOSE_WEIGHT), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&sc_address),
                endpoint: managed_buffer!(b"testendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"testperm1"), managed_buffer!(b"testperm2"), managed_buffer!(b"testperm3")]);

            sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b"content hash"),
                managed_buffer!(b"content signature"),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let proposal = sc.proposals(1).get();

            assert_eq!(0, proposal.starts_at);
            assert_eq!(10_800, longest_voting_period_minutes as u64 * 60);
        })
        .assert_ok();
}

#[test]
fn it_allows_anyone_to_propose_when_leaderless() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let sc_address = setup.contract.address_ref().clone();
    let proposer_address = setup.user_address.clone();

    setup.configure_gov_token(true);
    setup.configure_leaderless();

    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(MIN_PROPOSE_WEIGHT), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&sc_address),
                endpoint: managed_buffer!(b"testendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"testperm1"), managed_buffer!(b"testperm2"), managed_buffer!(b"testperm3")]);

            sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b"content hash"),
                managed_buffer!(b"content signature"),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();
}
