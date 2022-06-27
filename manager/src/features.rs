elrond_wasm::imports!();

use crate::config;

pub const FEATURE_NAME_LEADER: &[u8] = b"leader";

#[elrond_wasm::module]
pub trait FeaturesModule: config::ConfigModule {
    fn set_features(&self, entity_address: &ManagedAddress, features: ManagedVec<ManagedBuffer>) {
        self.features(&entity_address).clear();

        for feature in features.into_iter() {
            self.enable_feature(&entity_address, feature);
        }
    }

    fn enable_feature(&self, entity_address: &ManagedAddress, feature: ManagedBuffer) {
        self.features(&entity_address).insert(feature);
    }

    fn disable_feature(&self, entity_address: &ManagedAddress, feature: ManagedBuffer) {
        self.features(&entity_address).swap_remove(&feature);
    }

    #[view(getFeatures)]
    #[storage_mapper("features")]
    fn features(&self, entity_address: &ManagedAddress) -> UnorderedSetMapper<ManagedBuffer>;
}
