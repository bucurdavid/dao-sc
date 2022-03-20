elrond_wasm::imports!();

use super::config;

#[elrond_wasm::module]
pub trait FactoryModule: config::ConfigModule {
    fn create_entity(&self, token_id: &TokenIdentifier, initial_tokens: &BigUint) -> ManagedAddress {
        let template_contract = self.get_template_address();
        let vote_nft_token_id = self.vote_nft_token_id().get();

        let (address, _) = self
            .entity_contract_proxy(ManagedAddress::zero())
            .init(
                OptionalValue::Some(token_id.clone()),
                OptionalValue::Some(vote_nft_token_id.clone()),
                OptionalValue::Some(initial_tokens.clone()),
            )
            .deploy_from_source(&template_contract, self.get_deploy_code_metadata());

        require!(!address.is_zero(), "address is zero");

        address
    }

    fn upgrade_entity(&self, address: ManagedAddress) {
        let template_contract = self.get_template_address();

        self.entity_contract_proxy(address)
            .init(OptionalValue::None, OptionalValue::None, OptionalValue::None)
            .upgrade_from_source(&template_contract, self.get_deploy_code_metadata());
    }

    fn enable_entity_features(&self, address: &ManagedAddress, features_names: MultiValueEncoded<ManagedBuffer>) {
        self.entity_contract_proxy(address.clone())
            .enable_features(features_names)
            .execute_on_dest_context();
    }

    fn get_template_address(&self) -> ManagedAddress {
        require!(!self.entity_templ_address().is_empty(), "no template set");

        self.entity_templ_address().get()
    }

    fn get_deploy_code_metadata(&self) -> CodeMetadata {
        CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE | CodeMetadata::PAYABLE | CodeMetadata::PAYABLE_BY_SC
    }

    #[proxy]
    fn entity_contract_proxy(&self, to: ManagedAddress) -> entity::Proxy<Self::Api>;
}
