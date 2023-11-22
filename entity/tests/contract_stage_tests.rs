use entity::config::*;
use entity::contract::*;
use entity::governance::proposal::*;
use entity::permission::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_locks_the_contract_stage() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let contract_address = setup.contract.address_ref();

    setup
        .blockchain
        .execute_tx(contract_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.stage(&managed_address!(contract_address)).set(managed_buffer!(b"dummy_code"));

            sc.lock_contract_stage_endpoint(managed_address!(contract_address));

            assert!(sc.is_stage_locked(&managed_address!(contract_address)), "contract stage should be locked");
        })
        .assert_ok();
}

#[test]
fn it_unlocks_the_contract_stage() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let contract_address = setup.contract.address_ref();

    setup
        .blockchain
        .execute_tx(contract_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.stage(&managed_address!(contract_address)).set(managed_buffer!(b"dummy_code"));
            sc.stage_lock(&managed_address!(contract_address)).set(true);

            sc.unlock_contract_stage_endpoint(managed_address!(contract_address));

            assert!(!sc.is_stage_locked(&managed_address!(contract_address)), "contract stage should be unlocked");
            assert!(sc.stage(&managed_address!(contract_address)).is_empty(), "contract stage should be empty");
        })
        .assert_ok();
}

#[test]
fn it_fails_stage_contract_by_non_developer() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let non_dev_address = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup
        .blockchain
        .execute_tx(&non_dev_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.stage_contract_endpoint(managed_address!(&non_dev_address), managed_buffer!(b"dummy_code"));
        })
        .assert_user_error("caller must be developer");
}

#[test]
fn it_stages_contract_and_creates_proposal_when_caller_has_permission() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let dev_address = setup.blockchain.create_user_account(&rust_biguint!(0));
    let contract = setup.blockchain.create_sc_account(&rust_biguint!(0), Option::None, entity::contract_obj, "");
    let contract_address = contract.address_ref().clone();

    setup
        .blockchain
        .execute_tx(&dev_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(ROLE_BUILTIN_DEVELOPER));
            sc.assign_role(managed_address!(&dev_address), managed_buffer!(ROLE_BUILTIN_DEVELOPER));

            sc.create_permission(
                managed_buffer!(b"activateSc"),
                managed_biguint!(0),
                managed_address!(&contract_address),
                managed_buffer!(b"stageContractAndPropose"),
                ManagedVec::new(),
                ManagedVec::new(),
            );

            sc.create_policy(
                managed_buffer!(ROLE_BUILTIN_DEVELOPER),
                managed_buffer!(b"activateSc"),
                PolicyMethod::Quorum,
                managed_biguint!(1u64),
                1,
            );

            let action = Action::<DebugApi> {
                destination: managed_address!(&contract_address),
                endpoint: managed_buffer!(b"stageContractAndPropose"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            };

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(vec![action]));
            let permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"activateSc")]);

            sc.stage_contract_and_propose_endpoint(
                managed_address!(&contract_address),
                managed_buffer!(b"new_code"),
                managed_buffer!(b"trusted_host_id"),
                managed_buffer!(b"content_hash"),
                managed_buffer!(b"content_sig"),
                actions_hash,
                permissions,
            );

            assert!(sc.stage_lock(&managed_address!(&contract_address)).get(), "stage should be locked");
            assert!(!sc.stage(&managed_address!(&contract_address)).is_empty(), "stage should not be empty");
        })
        .assert_ok();
}

#[test]
fn it_cancels_a_previously_created_activation_proposal_when_exists() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let dev_address = setup.blockchain.create_user_account(&rust_biguint!(0));
    let contract = setup.blockchain.create_sc_account(&rust_biguint!(0), Option::None, entity::contract_obj, "");
    let contract_address = contract.address_ref().clone();

    setup
        .blockchain
        .execute_tx(&dev_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(ROLE_BUILTIN_DEVELOPER));
            sc.assign_role(managed_address!(&dev_address), managed_buffer!(ROLE_BUILTIN_DEVELOPER));

            sc.create_permission(
                managed_buffer!(b"activateSc"),
                managed_biguint!(0),
                managed_address!(&contract_address),
                managed_buffer!(b"stageContractAndPropose"),
                ManagedVec::new(),
                ManagedVec::new(),
            );

            sc.create_policy(
                managed_buffer!(ROLE_BUILTIN_DEVELOPER),
                managed_buffer!(b"activateSc"),
                PolicyMethod::Quorum,
                managed_biguint!(2u64),
                1,
            );

            // first activation proposal
            let action = Action::<DebugApi> {
                destination: managed_address!(&contract_address),
                endpoint: managed_buffer!(b"stageContractAndPropose"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            };

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(vec![action]));
            let permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"activateSc")]);

            let first_proposal_id = sc.stage_contract_and_propose_endpoint(
                managed_address!(&contract_address),
                managed_buffer!(b"new_code"),
                managed_buffer!(b"trusted_host_id1"),
                managed_buffer!(b"content_hash"),
                managed_buffer!(b"content_sig"),
                actions_hash,
                permissions,
            );

            // second activation proposal with different code
            let action = Action::<DebugApi> {
                destination: managed_address!(&contract_address),
                endpoint: managed_buffer!(b"stageContractAndPropose"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            };

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(vec![action]));
            let permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"activateSc")]);

            let second_proposal_id = sc.stage_contract_and_propose_endpoint(
                managed_address!(&contract_address),
                managed_buffer!(b"some_other_code"),
                managed_buffer!(b"trusted_host_id2"),
                managed_buffer!(b"content_hash"),
                managed_buffer!(b"content_sig"),
                actions_hash,
                permissions,
            );

            let first_proposal = sc.proposals(first_proposal_id).get();
            assert_eq!(sc.get_proposal_status(&first_proposal), ProposalStatus::Canceled);

            let second_proposal = sc.proposals(second_proposal_id).get();
            assert_eq!(sc.get_proposal_status(&second_proposal), ProposalStatus::Active);

            assert_eq!(sc.stage_current_proposal(&managed_address!(&contract_address)).get(), second_proposal_id);
        })
        .assert_ok();
}

#[test]
fn it_fails_to_lock_stage_when_code_stage_is_empty() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let contract_address = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup
        .blockchain
        .execute_tx(&setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.lock_contract_stage_endpoint(managed_address!(&contract_address));
        })
        .assert_user_error("contract stage is empty");
}

#[test]
fn it_fails_to_lock_stage_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let contract_address = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.lock_contract_stage_endpoint(managed_address!(&contract_address));
        })
        .assert_user_error("action not allowed by user");
}

#[test]
fn it_fails_to_unlock_stage_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let contract_address = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.unlock_contract_stage_endpoint(managed_address!(&contract_address));
        })
        .assert_user_error("action not allowed by user");
}

#[test]
fn it_fails_to_stage_code_when_stage_is_locked() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let dev_address = setup.blockchain.create_user_account(&rust_biguint!(0));
    let contract_address = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup
        .blockchain
        .execute_tx(&dev_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(ROLE_BUILTIN_DEVELOPER));
            sc.assign_role(managed_address!(&dev_address), managed_buffer!(ROLE_BUILTIN_DEVELOPER));

            sc.stage_lock(&managed_address!(&contract_address)).set(true);

            sc.stage_contract_endpoint(managed_address!(&contract_address), managed_buffer!(b"dummy_code"));
        })
        .assert_user_error("contract stage is locked");
}

#[test]
fn it_fails_to_stage_code_when_code_is_empty() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let dev_address = setup.blockchain.create_user_account(&rust_biguint!(0));
    let contract = setup.blockchain.create_sc_account(&rust_biguint!(0), Option::None, entity::contract_obj, "");
    let contract_address = contract.address_ref().clone();

    setup
        .blockchain
        .execute_tx(&dev_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(ROLE_BUILTIN_DEVELOPER));
            sc.assign_role(managed_address!(&dev_address), managed_buffer!(ROLE_BUILTIN_DEVELOPER));

            let code = managed_buffer!(b"");

            sc.stage_contract_endpoint(managed_address!(&contract_address), code);
        })
        .assert_user_error("code must not be empty");
}

#[test]
fn it_fails_to_stage_code_when_address_not_a_contract_address() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let dev_address = setup.blockchain.create_user_account(&rust_biguint!(0));
    let user_address = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup
        .blockchain
        .execute_tx(&dev_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(ROLE_BUILTIN_DEVELOPER));
            sc.assign_role(managed_address!(&dev_address), managed_buffer!(ROLE_BUILTIN_DEVELOPER));

            sc.stage_contract_endpoint(managed_address!(&user_address), managed_buffer!(b"dummy_code"));
        })
        .assert_user_error("address must be contract");
}

#[test]
fn it_fails_to_stage_code_when_address_is_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let dev_address = setup.blockchain.create_user_account(&rust_biguint!(0));
    let contract_address = setup.contract.address_ref().clone();

    setup
        .blockchain
        .execute_tx(&dev_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(ROLE_BUILTIN_DEVELOPER));
            sc.assign_role(managed_address!(&dev_address), managed_buffer!(ROLE_BUILTIN_DEVELOPER));

            sc.stage_contract_endpoint(managed_address!(&contract_address), managed_buffer!(b"dummy_code"));
        })
        .assert_user_error("address must not be self");
}
