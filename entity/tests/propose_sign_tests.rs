use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::*;
use entity::permission::*;
use setup::*;

mod setup;

#[test]
fn it_signs_a_proposal_on_proposing_if_proposal_requires_signing() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let sc_address = setup.contract.address_ref().clone();
    let proposer_address = setup.user_address.clone();
    let mut proposal_id: u64 = 0;

    setup.configure_gov_token();

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
            sc.create_permission(
                managed_buffer!(b"testperm"),
                managed_biguint!(0),
                managed_address!(&sc_address),
                managed_buffer!(b"testendpoint"),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_policy(
                managed_buffer!(b"builder"),
                managed_buffer!(b"testperm"),
                PolicyMethod::Quorum,
                managed_biguint!(QURUM),
                VOTING_PERIOD_MINUTES_DEFAULT,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(
            &proposer_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
            |sc| {
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
                let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"testperm")]);

                proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), actions_hash, actions_permissions);
            },
        )
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let expected_signer_id = sc.users().get_user_id(&managed_address!(&proposer_address));

            assert!(sc.proposal_signers(proposal_id, &managed_buffer!(b"builder")).contains(&expected_signer_id));
        })
        .assert_ok();
}

#[test]
fn it_creates_a_proposal_without_esdt_payment_if_not_required() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let sc_address = setup.contract.address_ref();
    let proposer_address = setup.user_address;
    let mut proposal_id: u64 = 0;

    // fails if no roles set, therefore requires token weight
    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b""),
                managed_buffer!(b""),
                managed_buffer!(b""),
                MultiValueManagedVec::new(),
            );
        })
        .assert_user_error("insufficient vote weight");

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
            sc.create_permission(
                managed_buffer!(b"testperm"),
                managed_biguint!(0),
                managed_address!(sc_address),
                managed_buffer!(b"testendpoint"),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_policy(
                managed_buffer!(b"builder"),
                managed_buffer!(b"testperm"),
                PolicyMethod::Quorum,
                managed_biguint!(QURUM),
                VOTING_PERIOD_MINUTES_DEFAULT,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(sc_address),
                endpoint: managed_buffer!(b"testendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"testperm")]);

            proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), actions_hash, actions_permissions);
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let expected_signer_id = sc.users().get_user_id(&managed_address!(&proposer_address));

            assert!(sc.proposal_signers(proposal_id, &managed_buffer!(b"builder")).contains(&expected_signer_id));
        })
        .assert_ok();
}

#[test]
fn it_does_not_self_sign_a_proposal_if_proposer_does_not_have_any_roles() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.owner_address.clone();
    let mut proposal_id: u64 = 0;

    setup.configure_gov_token();

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(
            &proposer_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
            |sc| {
                proposal_id = sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    managed_buffer!(b""),
                    managed_buffer!(b""),
                    managed_buffer!(b""),
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert!(sc.proposal_signers(proposal_id, &managed_buffer!(b"builder")).is_empty());
        })
        .assert_ok();
}
