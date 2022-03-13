#![no_std]
#![feature(generic_associated_types)]

elrond_wasm::imports!();

pub mod esdt;
pub mod features;
pub mod governance;

#[elrond_wasm::contract]
pub trait Entity:
    features::FeaturesModule
    + esdt::EsdtModule
    + governance::GovernanceModule
    + governance::configurable::GovConfigurableModule
    + governance::storage::GovStorageModule
    + governance::events::GovEventsModule
{
    #[init]
    fn init(&self, #[var_args] opt_initial_config: OptionalValue<(TokenIdentifier, BigUint)>) {
        if let OptionalValue::Some((token_id, initial_tokens)) = opt_initial_config {
            self.token_id().set_if_empty(&token_id);
            self.init_governance_module(&token_id, &initial_tokens);
        }
    }

    #[endpoint(enableFeatures)]
    fn enable_features(&self, #[var_args] features: MultiValueEncoded<ManagedBuffer>) {
        for feature in &features.to_vec() {
            self.set_feature_flag(feature, true);
        }
    }
}
