use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::*;
use entity::governance::proposal::*;
use entity::permission::*;
use setup::*;

mod setup;

#[test]
fn it_signs_a_proposal() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let owner_address = &setup.owner_address;
    let signer_address = &setup.user_address;
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id: u64 = 0;

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.create_role(managed_buffer!(b"builder"));
        sc.create_permission(managed_buffer!(b"perm"), managed_address!(&action_receiver), managed_buffer!(b"myendpoint"), ManagedVec::new());
        sc.create_policy(managed_buffer!(b"builder"), managed_buffer!(b"perm"), PolicyMethod::Quorum, BigUint::from(1u64), 10);
        sc.assign_role(managed_address!(&owner_address), managed_buffer!(b"builder"));
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();
        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            endpoint: managed_buffer!(b"myendpoint"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            value: managed_biguint!(0),
            payments: ManagedVec::new(),
        });

        let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
        let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"perm")]);

        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), actions_hash, actions_permissions);
    })
    .assert_ok();

    setup.blockchain.execute_tx(&signer_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.sign_endpoint(proposal_id);

        assert_eq!(1, sc.proposal_signers(proposal_id, &managed_buffer!(b"builder")).len());
    })
    .assert_ok();
}
