#![no_std]
#![feature(generic_associated_types)]

use config::SEALED_ON;

elrond_wasm::imports!();

pub mod config;
pub mod features;
pub mod governance;

#[elrond_wasm::contract]
pub trait Entity:
    config::ConfigModule
    + features::FeaturesModule
    + governance::GovernanceModule
    + governance::events::GovEventsModule
    + governance::proposal::ProposalModule
    + governance::vote::VoteModule
{
    #[init]
    fn init(&self, #[var_args] opt_token: OptionalValue<TokenIdentifier>, #[var_args] opt_initial_tokens: OptionalValue<BigUint>) {
        if let (OptionalValue::Some(token_id), OptionalValue::Some(initial_tokens)) = (opt_token, opt_initial_tokens) {
            self.token().set_token_id(&token_id);
            self.init_governance_module(&token_id, &initial_tokens);
        }
    }

    #[endpoint(setFeatures)]
    fn set_features_endpoint(&self, #[var_args] features: MultiValueEncoded<MultiValue2<ManagedBuffer, ManagedBuffer>>) {
        self.require_caller_self_or_unsealed();

        for feature_setting in features.into_iter() {
            let (feature_name, feature_enabled_arg) = feature_setting.into_tuple();
            let is_enabled = if feature_enabled_arg == ManagedBuffer::from(b"true") {
                true
            } else {
                false
            };

            self.set_feature_flag(feature_name, is_enabled);
        }
    }

    #[payable("*")]
    #[endpoint(seal)]
    fn seal_endpoint(&self) {
        let caller = self.blockchain().get_caller();
        let proof = self.call_value().payment();

        self.require_not_sealed();
        require!(!self.vote_nft_token().is_empty(), "vote nft token must be set");
        require!(proof.token_identifier == self.token().get_token_id(), "invalid token proof");

        self.sealed().set(SEALED_ON);

        self.send()
            .direct(&caller, &proof.token_identifier, proof.token_nonce, &proof.amount, &[]);

        self.vote_nft_token()
            .set_local_roles(&[EsdtLocalRole::NftCreate, EsdtLocalRole::NftBurn][..], None);
    }

    #[view(getVersion)]
    fn version_view(&self) -> &'static [u8] {
        env!("CARGO_PKG_VERSION").as_bytes()
    }
}
