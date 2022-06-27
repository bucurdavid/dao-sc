elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait ConfigModule {
    fn require_entity_exists(&self, entity_address: &ManagedAddress) {
        require!(self.entities().contains(&entity_address), "entity does not exist");
    }

    fn get_template_address(&self) -> ManagedAddress {
        require!(!self.entity_templ_address().is_empty(), "no template set");

        self.entity_templ_address().get()
    }

    #[only_owner]
    #[endpoint(setDailyBaseCost)]
    fn set_daily_base_cost_endpoint(&self, amount: BigUint) {
        require!(amount > 0, "can not be zero");
        self.cost_base_daily_amount().set(amount);
    }

    #[only_owner]
    #[endpoint(setDailyFeatureCost)]
    fn set_daily_feature_cost_endpoint(&self, feature: ManagedBuffer, amount: BigUint) {
        require!(amount > 0, "can not be zero");
        self.cost_feature_daily_amount(&feature).set(amount);
    }

    #[storage_mapper("entities")]
    fn entities(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getEntityTemplateAddress)]
    #[storage_mapper("entity_templ_address")]
    fn entity_templ_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getTrustedHostAddress)]
    #[storage_mapper("trusted_host_addr")]
    fn trusted_host_address(&self) -> SingleValueMapper<ManagedAddress>;

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
