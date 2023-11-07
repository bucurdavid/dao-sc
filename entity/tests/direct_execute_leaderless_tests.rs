use entity::governance::proposal::*;
use entity::governance::*;
use entity::permission::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_directly_executes_an_action_unilaterally_with_permission() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.configure_gov_token(true);
    setup.configure_leaderless();

    setup.blockchain.set_egld_balance(setup.contract.address_ref(), &rust_biguint!(1));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"developer"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"developer"));
            sc.create_permission(
                managed_buffer!(b"sendEgld"),
                managed_biguint!(1),
                managed_address!(&action_receiver),
                ManagedBuffer::new(),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_policy(
                managed_buffer!(b"developer"),
                managed_buffer!(b"sendEgld"),
                PolicyMethod::One,
                BigUint::from(1u64),
                10,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(1),
                payments: ManagedVec::new(),
            });

            sc.direct_execute_endpoint(MultiValueManagedVec::from(actions));
        })
        .assert_ok();

    setup.blockchain.check_egld_balance(&action_receiver, &rust_biguint!(1));
}

#[test]
fn it_fails_when_caller_has_no_permissions() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.configure_gov_token(true);

    setup.blockchain.set_egld_balance(setup.contract.address_ref(), &rust_biguint!(1));

    // caller is not assigned to the developer role
    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"developer"));
            sc.create_permission(
                managed_buffer!(b"sendEgld"),
                managed_biguint!(1),
                managed_address!(&action_receiver),
                ManagedBuffer::new(),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_policy(
                managed_buffer!(b"developer"),
                managed_buffer!(b"sendEgld"),
                PolicyMethod::One,
                BigUint::from(1u64),
                10,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(1),
                payments: ManagedVec::new(),
            });

            sc.direct_execute_endpoint(MultiValueManagedVec::from(actions));
        })
        .assert_user_error("no permission for action");

    setup.blockchain.check_egld_balance(&action_receiver, &rust_biguint!(0));
}
