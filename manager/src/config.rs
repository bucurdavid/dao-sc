elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait ConfigModule {
    #[view(getEntityAddress)]
    fn get_entity_address_view(&self, token_id: TokenIdentifier) -> ManagedAddress {
        self.entities_map().get(&token_id).unwrap_or_default()
    }

    fn get_entity_address(&self, token_id: &TokenIdentifier) -> ManagedAddress {
        require!(self.entities_map().contains_key(&token_id), "entity does not exist");

        self.entities_map().get(&token_id).unwrap()
    }

    fn get_template_address(&self) -> ManagedAddress {
        require!(!self.entity_templ_address().is_empty(), "no template set");

        self.entity_templ_address().get()
    }

    #[storage_mapper("entities")]
    fn entities_map(&self) -> MapMapper<TokenIdentifier, ManagedAddress>;

    #[view(getEntityTemplateAddress)]
    #[storage_mapper("entity_templ_address")]
    fn entity_templ_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("cost_token")]
    fn cost_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("cost_creation_amount")]
    fn cost_creation_amount(&self) -> SingleValueMapper<BigUint>;

    #[view(getMinBoostAmount)]
    #[storage_mapper("cost_boost_min_amount")]
    fn cost_boost_min_amount(&self) -> SingleValueMapper<BigUint>;

    #[view(getBaseDailyCost)]
    #[storage_mapper("cost_base_daily_amount")]
    fn cost_base_daily_amount(&self) -> SingleValueMapper<BigUint>;

    #[view(getFeatureDailyCost)]
    #[storage_mapper("cost_feature_daily_amount")]
    fn cost_feature_daily_amount(&self, feature: &ManagedBuffer) -> SingleValueMapper<BigUint>;
}
