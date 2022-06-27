#![no_std]
#![feature(generic_associated_types)]

elrond_wasm::imports!();

use config::SEALED_ON;

pub mod config;
pub mod governance;
pub mod permission;

#[elrond_wasm::contract]
pub trait Entity:
    config::ConfigModule
    + permission::PermissionModule
    + governance::GovernanceModule
    + governance::events::GovEventsModule
    + governance::proposal::ProposalModule
    + governance::vote::VoteModule
{
    #[init]
    fn init(&self, trusted_host_address: ManagedAddress, opt_token: OptionalValue<TokenIdentifier>, opt_initial_tokens: OptionalValue<BigUint>, opt_leader: OptionalValue<ManagedAddress>) {
        self.init_permission_module(opt_leader);
        self.trusted_host_address().set(&trusted_host_address);

        if let (OptionalValue::Some(token_id), OptionalValue::Some(initial_tokens)) = (opt_token, opt_initial_tokens) {
            self.token().set_token_id(&token_id);
            self.init_governance_module(&token_id, &initial_tokens);
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

        // TODO: upgrade token to disallow transferring ownership & remove upgradability with controlChanges
    }

    #[view(getVersion)]
    fn version_view(&self) -> &'static [u8] {
        env!("CARGO_PKG_VERSION").as_bytes()
    }
}
