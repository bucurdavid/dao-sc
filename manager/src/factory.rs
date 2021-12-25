elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait FactoryModule {
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

    fn create_token_properties(&self, dec_amount: usize) -> FungibleTokenProperties {
        FungibleTokenProperties {
            num_decimals: dec_amount,
            can_burn: false,
            can_mint: false,
            can_freeze: true,
            can_wipe: true,
            can_pause: true,
            can_change_owner: false,
            can_upgrade: false,
            can_add_special_roles: true,
        }
    }

    #[storage_mapper("entity_templ_addr")]
    fn entity_templ_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn entity_contract_proxy(&self, to: ManagedAddress) -> entity::Proxy<Self::Api>;
}
