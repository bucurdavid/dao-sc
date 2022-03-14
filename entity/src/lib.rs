#![no_std]
#![feature(generic_associated_types)]

elrond_wasm::imports!();

pub mod features;
pub mod governance;

#[elrond_wasm::contract]
pub trait Entity:
    features::FeaturesModule
    + governance::GovernanceModule
    + governance::configurable::GovConfigurableModule
    + governance::storage::GovStorageModule
    + governance::events::GovEventsModule
{
    #[init]
    fn init(&self, #[var_args] opt_token: OptionalValue<TokenIdentifier>, #[var_args] opt_initial_tokens: OptionalValue<BigUint>) {
        if let (OptionalValue::Some(token_id), OptionalValue::Some(initial_tokens)) = (opt_token, opt_initial_tokens) {
            self.token().set_token_id(&token_id);
            self.init_governance_module(&token_id, &initial_tokens);
        }
    }

    #[endpoint(enableFeatures)]
    fn enable_features(&self, #[var_args] features: MultiValueEncoded<ManagedBuffer>) {
        for feature in &features.to_vec() {
            self.set_feature_flag(feature, true);
        }
    }

    #[view(getTokenId)]
    #[storage_mapper("token")]
    fn token(&self) -> FungibleTokenMapper<Self::Api>;
}
