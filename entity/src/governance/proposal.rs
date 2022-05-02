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
    MultiValue8<ManagedAddress<M>, ManagedBuffer<M>, u64, TokenIdentifier<M>, u64, BigUint<M>, usize, MultiValueManagedVec<M, ManagedBuffer<M>>>;

impl<M: ManagedTypeApi> Action<M> {
    pub fn into_multiarg(self) -> ActionAsMultiArg<M> {
        (
            self.address,
            self.endpoint,
            self.gas_limit,
            self.token_id,
            self.token_nonce,
            self.amount,
            self.arguments.len(),
            MultiValueManagedVec::from(self.arguments),
        )
            .into()
    }
}

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Debug)]
pub enum ProposalStatus {
    Pending,
    Active,
    Defeated,
    Succeeded,
    Executed,
}

#[elrond_wasm::module]
pub trait ProposalModule: config::ConfigModule {
    fn create_proposal(
        &self,
        title: ManagedBuffer,
        description: ManagedBuffer,
        actions: ManagedVec<Action<Self::Api>>,
        vote_weight: BigUint,
    ) -> Proposal<Self::Api> {
        let proposer = self.blockchain().get_caller();
        let proposal_id = self.next_proposal_id().get();
        let starts_at = self.blockchain().get_block_timestamp();
        let voting_period_minutes = self.voting_period_in_minutes().get() as u64;
        let ends_at = starts_at + voting_period_minutes * 60;

        require!(vote_weight >= self.min_proposal_vote_weight().get(), "insufficient vote weight");

        let proposal = Proposal {
            id: proposal_id.clone(),
            proposer: proposer.clone(),
            title,
            description,
            starts_at,
            ends_at,
            was_executed: false,
            actions,
            votes_for: vote_weight.clone(),
            votes_against: BigUint::zero(),
        };

        self.proposals(proposal_id.clone()).set(&proposal);
        self.next_proposal_id().set(proposal_id + 1);

        proposal
    }

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
        let vote_for_percent = &proposal.votes_for * &BigUint::from(100u64) / &total_votes;
        let vote_for_percent_to_pass = BigUint::from(50u64);

        if vote_for_percent > vote_for_percent_to_pass && &proposal.votes_for >= &quorum {
            ProposalStatus::Succeeded
        } else {
            ProposalStatus::Defeated
        }
    }

    fn execute_proposal(&self, proposal: &Proposal<Self::Api>) {
        let gov_token_id = self.governance_token_id().get();

        for action in proposal.actions.iter() {
            let mut call = self
                .send()
                .contract_call::<()>(action.address, action.endpoint)
                .with_gas_limit(action.gas_limit);

            if action.amount > 0 {
                call = if action.token_id == TokenIdentifier::egld() {
                    call.with_egld_transfer(action.amount)
                } else {
                    if action.token_id == gov_token_id {
                        self.require_governance_tokens_available(&action.amount);
                    }

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
