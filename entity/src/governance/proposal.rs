elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::config;

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct Proposal<M: ManagedTypeApi> {
    pub id: u64,
    pub proposer: ManagedAddress<M>,
    pub title: ManagedBuffer<M>,
    pub description: ManagedBuffer<M>,
    pub starts_at: u64,
    pub ends_at: u64,
    pub was_executed: bool,
    pub actions: ManagedVec<M, Action<M>>,
    pub votes_for: BigUint<M>,
    pub votes_against: BigUint<M>,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem)]
pub struct Action<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub endpoint: ManagedBuffer<M>,
    pub arguments: ManagedVec<M, ManagedBuffer<M>>,
    pub gas_limit: u64,
    pub token_id: TokenIdentifier<M>,
    pub token_nonce: u64,
    pub amount: BigUint<M>,
}

pub type ActionAsMultiArg<M> =
    MultiValue7<u64, ManagedAddress<M>, TokenIdentifier<M>, u64, BigUint<M>, ManagedBuffer<M>, ManagedVec<M, ManagedBuffer<M>>>;

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Active,
    Defeated,
    Succeeded,
    Executed,
}

#[elrond_wasm::module]
pub trait ProposalModule: config::ConfigModule {
    fn get_proposal_status(&self, proposal: &Proposal<Self::Api>) -> ProposalStatus {
        if proposal.was_executed {
            return ProposalStatus::Executed;
        }

        let current_time = self.blockchain().get_block_timestamp();

        if current_time >= proposal.starts_at && current_time < proposal.ends_at {
            return ProposalStatus::Active;
        }

        let quorum = self.quorum().get();
        let total_votes = &proposal.votes_for + &proposal.votes_against;
        let vote_for_percent = &proposal.votes_for / &total_votes * &BigUint::from(100u64);
        let vote_for_percent_to_pass = BigUint::from(50u64);

        if vote_for_percent > vote_for_percent_to_pass && &proposal.votes_for >= &quorum {
            ProposalStatus::Succeeded
        } else {
            ProposalStatus::Defeated
        }
    }

    fn execute_proposal(&self, proposal: &Proposal<Self::Api>) {
        for action in proposal.actions.iter() {
            let mut call = self
                .send()
                .contract_call::<()>(action.address, action.endpoint)
                .with_gas_limit(action.gas_limit);

            if action.amount > 0 {
                call = if action.token_id == TokenIdentifier::egld() {
                    call.with_egld_transfer(action.amount)
                } else {
                    call.add_token_transfer(action.token_id, action.token_nonce, action.amount)
                }
            }

            call.transfer_execute()
        }
    }

    fn proposal_exists(&self, proposal_id: u64) -> bool {
        !self.proposals(proposal_id).is_empty()
    }
}
