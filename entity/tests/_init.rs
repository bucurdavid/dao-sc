elrond_wasm::imports!();

use entity::*;
use entity::esdt::*;
use elrond_wasm_debug::{testing_framework::*, DebugApi, managed_token_id, rust_biguint };

pub const ENTITY_TOKEN_ID: &[u8] = b"SUPER-abcdef";

const WASM_PATH: &'static str = "output/entity.wasm";

#[allow(dead_code)]
struct EntitySetup<ObjBuilder>
where
    ObjBuilder: 'static + Copy + Fn(DebugApi) -> entity::ContractObj<DebugApi>,
{
    pub blockchain: BlockchainStateWrapper,
    pub owner_address: Address,
    pub contract: ContractObjWrapper<entity::ContractObj<DebugApi>, ObjBuilder>,
}

fn setup_entity<ObjBuilder>(
    builder: ObjBuilder,
) -> EntitySetup<ObjBuilder>
where
    ObjBuilder: 'static + Copy + Fn(DebugApi) -> entity::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain = BlockchainStateWrapper::new();
    let owner_address = blockchain.create_user_account(&rust_zero);
    let contract = blockchain.create_sc_account(
        &rust_zero,
        Some(&owner_address),
        builder,
        WASM_PATH,
    );

    blockchain.execute_tx(&owner_address, &contract, &rust_zero, |sc| {
        let entity_token_id = managed_token_id!(ENTITY_TOKEN_ID);

        sc.init(OptionalArg::Some(entity_token_id));

        StateChange::Commit
    });

    EntitySetup {
        blockchain,
        owner_address,
        contract,
    }
}

#[test]
fn it_initializes_the_contract() {
    let mut setup = setup_entity(entity::contract_obj);

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(managed_token_id!(ENTITY_TOKEN_ID), sc.token_id().get());
    });
}
