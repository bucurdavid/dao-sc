elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait FactoryModule {
    fn init_factory_module(&self, entity_template_address: ManagedAddress) {
        self.entity_templ_address().set(&entity_template_address);
    }

    fn create_entity(&self) -> SCResult<ManagedAddress> {
        require!(!self.entity_templ_address().is_empty(), "no template set");

        let template_contract = self.entity_templ_address().get();

        let (address, _) = self
            .entity_contract_proxy(ManagedAddress::zero())
            .init()
            .deploy_from_source(&template_contract, CodeMetadata::UPGRADEABLE);

        require!(!address.is_zero(), "address is zero");

        Ok(address)
    }

    fn issue_token(&self, name: ManagedBuffer, ticker: ManagedBuffer, dec_amount: usize, cost: BigUint) -> AsyncCall {
        let properties = FungibleTokenProperties {
            num_decimals: dec_amount,
            can_burn: false,
            can_mint: false,
            can_freeze: true,
            can_wipe: true,
            can_pause: true,
            can_change_owner: false,
            can_upgrade: false,
            can_add_special_roles: true,
        };

        self.send()
            .esdt_system_sc_proxy()
            .issue_fungible(cost, &name, &ticker, &BigUint::from(0u32), properties)
            .async_call()
    }

    fn set_entity_edst_roles(&self, token_id: &TokenIdentifier, entity_address: &ManagedAddress) -> AsyncCall {
        let roles = [EsdtLocalRole::Mint, EsdtLocalRole::Burn];

        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(&entity_address, &token_id, (&roles[..]).into_iter().cloned())
            .async_call()
    }

    fn transfer_entity_edst_ownership(&self, token_id: &TokenIdentifier, entity_address: &ManagedAddress) -> AsyncCall {
        self.send()
            .esdt_system_sc_proxy()
            .transfer_ownership(&token_id, &entity_address.to_address())
            .async_call()
    }

    #[storage_mapper("entity_templ_addr")]
    fn entity_templ_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn entity_contract_proxy(&self, to: ManagedAddress) -> entity::Proxy<Self::Api>;
}
