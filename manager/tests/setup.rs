elrond_wasm::imports!();

use elrond_wasm_debug::{managed_address, managed_biguint, managed_token_id, rust_biguint, testing_framework::*, DebugApi};
use manager::config::*;
use manager::*;

pub const SUPER_TOKEN_ID: &[u8] = b"SUPER-abcdef";
pub const WASM_PATH: &'static str = "output/manager.wasm";
pub const WASM_PATH_ENTITY_TEMPLATE: &'static str = "output/entity.wasm";

#[allow(dead_code)]
pub struct ManagerSetup<ObjBuilder>
where
    ObjBuilder: 'static + Copy + Fn() -> manager::ContractObj<DebugApi>,
{
    pub blockchain: BlockchainStateWrapper,
    pub owner_address: Address,
    pub contract: ContractObjWrapper<manager::ContractObj<DebugApi>, ObjBuilder>,
}

pub fn setup_manager<ObjBuilder>(builder: ObjBuilder) -> ManagerSetup<ObjBuilder>
where
    ObjBuilder: 'static + Copy + Fn() -> manager::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain = BlockchainStateWrapper::new();
    let owner_address = blockchain.create_user_account(&rust_zero);
    let contract = blockchain.create_sc_account(&rust_zero, Some(&owner_address), builder, WASM_PATH);
    let contract_entity_template = blockchain.create_sc_account(&rust_zero, Some(&owner_address), builder, WASM_PATH_ENTITY_TEMPLATE);

    blockchain.set_esdt_balance(&owner_address, SUPER_TOKEN_ID, &rust_biguint!(10_000));

    blockchain
        .execute_tx(&owner_address, &contract, &rust_zero, |sc| {
            sc.init(
                managed_address!(contract_entity_template.address_ref()),
                managed_token_id!(SUPER_TOKEN_ID),
                managed_biguint!(500),
            );
        })
        .assert_ok();

    ManagerSetup {
        blockchain,
        owner_address,
        contract,
    }
}

#[test]
fn it_initializes_the_contract() {
    let mut setup = setup_manager(manager::contract_obj);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(managed_token_id!(SUPER_TOKEN_ID), sc.cost_token_id().get());
        })
        .assert_ok();
}
