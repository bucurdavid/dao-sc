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
    + governance::token::TokenModule
{
    #[init]
    fn init(&self, trusted_host_address: ManagedAddress, opt_leader: OptionalValue<ManagedAddress>) {
        self.trusted_host_address().set(&trusted_host_address);
        self.init_governance_module();

        if let OptionalValue::Some(leader) = opt_leader {
            self.init_permission_module(leader);
        }
    }

    #[endpoint(seal)]
    fn seal_endpoint(&self) {
        self.require_not_sealed();
        self.require_caller_has_leader_role();
        self.sealed().set(SEALED_ON);
    }

    #[view(getVersion)]
    fn version_view(&self) -> &'static [u8] {
        env!("CARGO_PKG_VERSION").as_bytes()
    }
}
