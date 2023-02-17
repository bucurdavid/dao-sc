multiversx_sc::imports!();

use crate::config::{self, UserId};

#[multiversx_sc::module]
pub trait PlugModule: config::ConfigModule {
    #[view(hasUserPlugVoted)]
    fn has_user_plug_voted_view(&self, proposal_id: u64, address: ManagedAddress) -> bool {
        self.has_user_plug_voted(proposal_id, &address)
    }

    fn is_plugged(&self) -> bool {
        !self.plug_sc_address().is_empty()
    }

    fn call_plug_vote_weight_async(&self) -> AsyncCall {
        require!(self.is_plugged(), "not plugged");

        let caller = self.blockchain().get_caller();

        self.plug_proxy(self.plug_sc_address().get()).get_dao_vote_weight_view(caller).async_call()
    }

    fn record_plug_vote(&self, voter: ManagedAddress, proposal_id: u64) {
        let user_id = self.users().get_user_id(&voter);

        self.plug_votes(proposal_id).insert(user_id);
    }

    fn has_user_plug_voted(&self, proposal_id: u64, address: &ManagedAddress) -> bool {
        let user_id = self.users().get_user_id(&address);

        self.plug_votes(proposal_id).contains(&user_id)
    }

    #[view(getPlugScAddress)]
    #[storage_mapper("plug_sc_addr")]
    fn plug_sc_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("plug_votes")]
    fn plug_votes(&self, proposal_id: u64) -> UnorderedSetMapper<UserId>;

    #[proxy]
    fn plug_proxy(&self, to: ManagedAddress) -> plug_proxy::Proxy<Self::Api>;
}

mod plug_proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait EntityPlugContractProxy {
        #[view(getDaoVoteWeight)]
        fn get_dao_vote_weight_view(&self, address: ManagedAddress) -> BigUint;
    }
}
