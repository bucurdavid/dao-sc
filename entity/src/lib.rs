#![no_std]

elrond_wasm::imports!();

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
    + governance::token::TokenModule
{
    #[init]
    fn init(&self, trusted_host_address: ManagedAddress, opt_leader: OptionalValue<ManagedAddress>) {
        self.trusted_host_address().set(&trusted_host_address);
        self.init_governance_module();

        if let OptionalValue::Some(leader) = opt_leader {
            self.init_permission_module(leader);
        }

        // Upgrade (idempotency is given)
        // TODO: remove after upgraded
        if !self.gov_token_id().is_empty() {
            let gov_token_id = self.gov_token_id().get();

            // Set true only here - it's done also on gov token init, but we need to do it for already configured daos too
            self.lock_vote_tokens(&gov_token_id).set(true);

            let old_protected = self.protected_vote_tokens(&gov_token_id).get();
            self.guarded_vote_tokens(&gov_token_id, 0).set_if_empty(old_protected);
        }
        // end upgrade
    }

    #[payable("EGLD")]
    #[endpoint(registerDns)]
    fn register_dns(&self, dns_address: ManagedAddress, name: ManagedBuffer) {
        self.require_caller_self();
        let payment = self.call_value().egld_value();

        self.dns_proxy(dns_address).register(&name).with_egld_transfer(payment).async_call().call_and_exit()
    }

    #[view(getVersion)]
    fn version_view(&self) -> &'static [u8] {
        env!("CARGO_PKG_VERSION").as_bytes()
    }

    #[proxy]
    fn dns_proxy(&self, to: ManagedAddress) -> dns_proxy::Proxy<Self::Api>;
}

mod dns_proxy {
    elrond_wasm::imports!();

    #[elrond_wasm::proxy]
    pub trait Dns {
        #[payable("EGLD")]
        #[endpoint]
        fn register(&self, name: &ManagedBuffer);
    }
}
