#![no_std]

elrond_wasm::imports!();

mod esdt;
mod features;

use features::{FeatureName, FEATURE_ON};

#[elrond_wasm::contract]
pub trait Entity: features::FeaturesModule + esdt::EsdtModule {
    #[init]
    fn init(&self, token_id: TokenIdentifier) {
        self.token_id().set(&token_id);
    }

    #[endpoint(enableFeatures)]
    fn enable_features(&self, #[var_args] features: VarArgs<Vec<u8>>) -> SCResult<()> {
        for feature in features.iter() {
            self.set_feature_flag(FeatureName(feature), FEATURE_ON);
        }

        Ok(())
    }
}
