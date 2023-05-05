multiversx_sc::imports!();

use crate::config::{self, UserId};

#[multiversx_sc::module]
pub trait PlugModule: config::ConfigModule {
    #[view(hasUserPlugVoted)]
    fn has_user_plug_voted_view(&self, proposal_id: u64, address: ManagedAddress) -> bool {
        self.has_user_plug_voted(proposal_id, &address)
    }

    fn is_plugged(&self) -> bool {
        !self.plug_contract().is_empty()
    }

    fn call_plug_vote_weight_async(&self) -> AsyncCall {
        require!(self.is_plugged(), "not plugged");

        let caller = self.blockchain().get_caller();
        let token = if self.gov_token_id().is_empty() {
            OptionalValue::None
        } else {
            OptionalValue::Some(self.gov_token_id().get())
        };

        self.plug_proxy(self.plug_contract().get()).get_dao_vote_weight_view(caller, token).async_call()
    }

    fn record_plug_vote(&self, voter: ManagedAddress, proposal_id: u64) {
        let user_id = self.users().get_or_create_user(&voter);

        self.plug_votes(proposal_id).insert(user_id);
    }

    fn has_user_plug_voted(&self, proposal_id: u64, address: &ManagedAddress) -> bool {
        let user_id = self.users().get_user_id(&address);

        if user_id == 0 {
            return false;
        }

        self.plug_votes(proposal_id).contains(&user_id)
    }

    #[view(getPlug)]
    fn get_plug_view(&self) -> MultiValue2<ManagedAddress, u8> {
        let plug_contract = self.plug_contract().get();
        let weight_decimals = self.plug_weight_decimals().get();

        (plug_contract, weight_decimals).into()
    }

    #[storage_mapper("plug:contract")]
    fn plug_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("plug:weight_decimals")]
    fn plug_weight_decimals(&self) -> SingleValueMapper<u8>;

    #[storage_mapper("plug:votes")]
    fn plug_votes(&self, proposal_id: u64) -> UnorderedSetMapper<UserId>;

    #[proxy]
    fn plug_proxy(&self, to: ManagedAddress) -> plug_proxy::Proxy<Self::Api>;
}

mod plug_proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait EntityPlugContractProxy {
        #[view(getDaoVoteWeight)]
        fn get_dao_vote_weight_view(&self, address: ManagedAddress, token: OptionalValue<TokenIdentifier>) -> BigUint;
    }
}
