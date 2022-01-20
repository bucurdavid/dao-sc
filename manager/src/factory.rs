elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait FactoryModule {
    fn init_factory_module(&self, entity_template_address: ManagedAddress) {
        self.entity_templ_address().set(&entity_template_address);
    }

    fn create_entity(&self, token_id: &TokenIdentifier) -> SCResult<ManagedAddress> {
        let template_contract = self.get_template_address()?;

        let (address, _) = self
            .entity_contract_proxy(ManagedAddress::zero())
            .init(OptionalArg::Some(token_id.clone()))
            .deploy_from_source(&template_contract, CodeMetadata::UPGRADEABLE);

        require!(!address.is_zero(), "address is zero");

        Ok(address)
    }

    fn upgrade_entity(&self, address: &ManagedAddress) -> SCResult<()> {
        let template_contract = self.get_template_address()?;

        self.entity_contract_proxy(address.clone())
            .init(OptionalArg::None)
            .upgrade_from_source(&template_contract, CodeMetadata::UPGRADEABLE);

        Ok(())
    }

    fn enable_entity_features(&self, address: &ManagedAddress, features_names: VarArgs<ManagedBuffer>) {
        self.entity_contract_proxy(address.clone())
            .enable_features(features_names)
            .execute_on_dest_context();
    }

    fn get_template_address(&self) -> SCResult<ManagedAddress> {
        require!(!self.entity_templ_address().is_empty(), "no template set");
        Ok(self.entity_templ_address().get())
    }

    #[storage_mapper("entity_templ_addr")]
    fn entity_templ_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn entity_contract_proxy(&self, to: ManagedAddress) -> entity::Proxy<Self::Api>;
}
