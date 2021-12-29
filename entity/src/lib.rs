#![no_std]

elrond_wasm::imports!();

mod esdt;
mod features;

use features::{FeatureName, FEATURE_ON};

#[elrond_wasm::contract]
pub trait Entity: features::FeaturesModule + esdt::EsdtModule {
    #[init]
    fn init(&self, token_id: TokenIdentifier) {
        self.token_id().set_if_empty(&token_id);
    }

    #[endpoint(enableFeatures)]
    fn enable_features(&self, #[var_args] features: VarArgs<ManagedBuffer>) -> SCResult<()> {
        for feature in features.iter() {
            self.set_feature_flag(FeatureName(feature.to_boxed_bytes().as_slice()), FEATURE_ON);
        }

        Ok(())
    }
}
