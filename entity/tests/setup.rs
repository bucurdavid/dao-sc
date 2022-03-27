elrond_wasm::imports!();

use elrond_wasm_debug::{managed_biguint, managed_token_id, rust_biguint, testing_framework::*, DebugApi};
use entity::config::*;
use entity::*;

pub const ENTITY_TOKEN_ID: &[u8] = b"SUPER-abcdef";
pub const VOTE_NFT_TOKEN_ID: &[u8] = b"SUPERVOTE-abcdef";
pub const WASM_PATH: &'static str = "output/entity.wasm";

#[allow(dead_code)]
pub struct EntitySetup<ObjBuilder>
where
    ObjBuilder: 'static + Copy + Fn() -> entity::ContractObj<DebugApi>,
{
    pub blockchain: BlockchainStateWrapper,
    pub owner_address: Address,
    pub contract: ContractObjWrapper<entity::ContractObj<DebugApi>, ObjBuilder>,
}

pub fn setup_entity<ObjBuilder>(builder: ObjBuilder) -> EntitySetup<ObjBuilder>
where
    ObjBuilder: 'static + Copy + Fn() -> entity::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain = BlockchainStateWrapper::new();
    let owner_address = blockchain.create_user_account(&rust_zero);
    let contract = blockchain.create_sc_account(&rust_zero, Some(&owner_address), builder, WASM_PATH);
    let initial_tokens = 1000u64;

    blockchain.set_esdt_balance(&owner_address, ENTITY_TOKEN_ID, &rust_biguint!(initial_tokens.clone()));

    blockchain
        .execute_tx(&owner_address, &contract, &rust_zero, |sc| {
            sc.init(
                OptionalValue::Some(managed_token_id!(ENTITY_TOKEN_ID)),
                OptionalValue::Some(managed_biguint!(initial_tokens)),
            );
        })
        .assert_ok();

    let vote_nft_roles = [EsdtLocalRole::NftCreate, EsdtLocalRole::NftBurn, EsdtLocalRole::NftUpdateAttributes];
    blockchain.set_esdt_local_roles(contract.address_ref(), VOTE_NFT_TOKEN_ID, &vote_nft_roles[..]);

    EntitySetup {
        blockchain,
        owner_address,
        contract,
    }
}

#[test]
fn it_initializes_the_contract() {
    let mut setup = setup_entity(entity::contract_obj);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(managed_token_id!(ENTITY_TOKEN_ID), sc.token().get_token_id());
        })
        .assert_ok();
}
