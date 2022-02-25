#![no_std]

elrond_wasm::imports!();

pub mod config;
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
    fn init(&self, #[var_args] opt_token_id: OptionalValue<TokenIdentifier>) -> SCResult<()> {
        if let OptionalValue::Some(token_id) = opt_token_id {
            self.token_id().set_if_empty(&token_id);
            self.init_governance_module(&token_id)?;
        }

        Ok(())
    }

    #[endpoint(enableFeatures)]
    fn enable_features(&self, #[var_args] features: ManagedVarArgs<ManagedBuffer>) -> SCResult<()> {
        for feature in &features.to_vec() {
            self.set_feature_flag(feature, true);
        }

        Ok(())
    }
}
