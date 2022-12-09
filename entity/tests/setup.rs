elrond_wasm::imports!();

use elrond_wasm_debug::testing_framework::*;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::GovernanceModule;
use entity::*;

pub const ENTITY_GOV_TOKEN_ID: &[u8] = b"SUPER-abcdef";
pub const ENTITY_GOV_TOKEN_SUPPLY: u64 = 1_000;
pub const ENTITY_FAKE_TOKEN_ID: &[u8] = b"FAKE-abcdef";
pub const MIN_WEIGHT_FOR_PROPOSAL: u64 = 2;
pub const POLL_DEFAULT_ID: u8 = 0;
pub const QURUM: u64 = 50;
pub const WASM_PATH: &'static str = "output/entity.wasm";

#[allow(dead_code)]
pub struct EntitySetup<ObjBuilder>
where
    ObjBuilder: 'static + Copy + Fn() -> entity::ContractObj<DebugApi>,
{
    pub blockchain: BlockchainStateWrapper,
    pub owner_address: Address,
    pub user_address: Address,
    pub trusted_host_address: Address,
    pub contract: ContractObjWrapper<entity::ContractObj<DebugApi>, ObjBuilder>,
}

impl<ObjBuilder> EntitySetup<ObjBuilder>
where
    ObjBuilder: 'static + Copy + Fn() -> entity::ContractObj<DebugApi>,
{
    pub fn new(builder: ObjBuilder) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let mut blockchain = BlockchainStateWrapper::new();
        let owner_address = blockchain.create_user_account(&rust_zero);
        let user_address = blockchain.create_user_account(&rust_biguint!(1000));
        let trusted_host_address = blockchain.create_user_account(&rust_zero);
        let contract = blockchain.create_sc_account(&rust_biguint!(100), Some(&owner_address), builder, WASM_PATH);

        blockchain.set_esdt_balance(&owner_address, ENTITY_GOV_TOKEN_ID, &rust_biguint!(ENTITY_GOV_TOKEN_SUPPLY));
        blockchain.set_esdt_balance(&user_address, ENTITY_GOV_TOKEN_ID, &rust_biguint!(ENTITY_GOV_TOKEN_SUPPLY));
        blockchain.set_esdt_balance(&user_address, ENTITY_FAKE_TOKEN_ID, &rust_biguint!(ENTITY_GOV_TOKEN_SUPPLY));

        blockchain
            .execute_tx(&owner_address, &contract, &rust_zero, |sc| {
                sc.init(managed_address!(&trusted_host_address), OptionalValue::Some(managed_address!(&owner_address)));

                // disable trusted host for tests
                sc.trusted_host_address().clear();
            })
            .assert_ok();

        Self {
            blockchain,
            owner_address,
            user_address,
            trusted_host_address,
            contract,
        }
    }

    pub fn configure_gov_token(&mut self, lock_vote_tokens: bool) {
        self.blockchain
            .execute_tx(&self.owner_address, &self.contract, &rust_biguint!(0), |sc| {
                sc.configure_governance_token(managed_token_id!(ENTITY_GOV_TOKEN_ID), managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL), lock_vote_tokens);

                // override defaults
                sc.quorum().set(managed_biguint!(QURUM));
                sc.min_propose_weight().set(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL));

                // assert
                assert_eq!(lock_vote_tokens, sc.lock_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID)).get());
            })
            .assert_ok();
    }
}

#[test]
fn it_initializes_the_contract() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_query(&setup.contract, |_| {
            //
        })
        .assert_ok();
}
