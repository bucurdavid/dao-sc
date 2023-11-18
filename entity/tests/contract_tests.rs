use entity::contract::*;
use entity::governance::proposal::*;
use entity::permission::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_locks_and_unlocks_contract_stage() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let contract_address = setup.contract.address_ref();

    // Locking the contract stage
    setup
        .blockchain
        .execute_tx(contract_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.lock_contract_stage_endpoint(managed_address!(contract_address));
            assert!(sc.is_stage_locked(&managed_address!(contract_address)), "contract stage should be locked");
        })
        .assert_ok();

    // Unlocking the contract stage
    setup
        .blockchain
        .execute_tx(contract_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.unlock_contract_stage_endpoint(managed_address!(contract_address));
            assert!(!sc.is_stage_locked(&managed_address!(contract_address)), "contract stage should be unlocked");
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
    let contract_address = setup.contract.address_ref().clone();
    let dev_address = setup.blockchain.create_user_account(&rust_biguint!(0));

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
                managed_address!(&dev_address),
                managed_buffer!(b"new_code"),
                managed_buffer!(b"trusted_host_id"),
                managed_buffer!(b"content_hash"),
                managed_buffer!(b"content_sig"),
                actions_hash,
                permissions,
            );

            assert!(sc.stage_lock(&managed_address!(&dev_address)).get(), "contract stage should be locked");
            assert!(!sc.stage(&managed_address!(&dev_address)).is_empty(), "contract should be staged");
        })
        .assert_ok();
}

#[test]
fn it_fails_activate_contract_when_no_code_staged() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let contract_address = setup.contract.address_ref();

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.activate_contract_endpoint(managed_address!(contract_address), CodeMetadata::DEFAULT, MultiValueEncoded::new());
        })
        .assert_user_error("contract not staged");
}
