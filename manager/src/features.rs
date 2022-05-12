elrond_wasm::imports!();

use crate::config;

#[elrond_wasm::module]
pub trait FeaturesModule: config::ConfigModule {
    fn set_features(&self, entity_token_id: &TokenIdentifier, features: MultiValueEncoded<MultiValue2<ManagedBuffer, ManagedBuffer>>) {
        for feature_setting in features.into_iter() {
            let (feature_name, feature_enabled_arg) = feature_setting.into_tuple();
            if feature_enabled_arg == ManagedBuffer::from(b"true") {
                self.enable_feature(&entity_token_id, feature_name);
            } else {
                self.disable_feature(&entity_token_id, feature_name);
            };
        }
    }

    fn enable_feature(&self, entity_token_id: &TokenIdentifier, feature: ManagedBuffer) {
        self.features(&entity_token_id).insert(feature);
    }

    fn disable_feature(&self, entity_token_id: &TokenIdentifier, feature: ManagedBuffer) {
        self.features(&entity_token_id).swap_remove(&feature);
    }

    #[view(getFeatures)]
    #[storage_mapper("features")]
    fn features(&self, entity_token_id: &TokenIdentifier) -> UnorderedSetMapper<ManagedBuffer>;
}
