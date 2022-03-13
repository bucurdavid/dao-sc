elrond_wasm::imports!();

pub mod configurable;
pub mod events;
pub mod storage;
pub mod types;

use types::*;

#[elrond_wasm::module]
pub trait GovernanceModule: configurable::GovConfigurableModule + storage::GovStorageModule + events::GovEventsModule {
    #[endpoint(withdrawVoteTokens)]
    fn withdraw_gov_tokens_endpoint(&self, proposal_id: usize) {
        self.require_valid_proposal_id(proposal_id);
        require!(
            self.get_proposal_status(proposal_id) != ProposalStatus::Active,
            "proposal is still active"
        );

        let caller = self.blockchain().get_caller();
        let gov_token_id = self.governance_token_id().get();
        let nr_votes_tokens = self.upvotes(proposal_id).get(&caller).unwrap_or_default();
        let nr_downvotes_tokens = self.downvotes(proposal_id).get(&caller).unwrap_or_default();
        let total_tokens = nr_votes_tokens + nr_downvotes_tokens;

        if total_tokens > 0 {
            self.upvotes(proposal_id).remove(&caller);
            self.downvotes(proposal_id).remove(&caller);
            self.send().direct(&caller, &gov_token_id, 0, &total_tokens, &[]);
        }
    }

    #[payable("*")]
    #[endpoint(propose)]
    fn propose_endpoint(
        &self,
        #[payment_amount] payment_amount: BigUint,
        title: ManagedBuffer,
        description: ManagedBuffer,
        #[var_args] actions: MultiValueManagedVec<Action<Self::Api>>,
    ) -> usize {
        self.require_payment_token_governance_token();
        require!(payment_amount >= self.min_token_balance_for_proposing().get(), "not enough tokens");

        let proposer = self.blockchain().get_caller();
        let proposal_id = self.proposals().len() + 1;
        let starts_at = self.blockchain().get_block_timestamp();
        let voting_period_hours = self.voting_period_in_hours().get() as u64;
        let ends_at = starts_at + voting_period_hours * 60 * 60;

        self.emit_proposal_created_event(proposal_id, &proposer, starts_at, ends_at, &title, &description);

        let _ = self.proposals().push(&Proposal {
            proposer: proposer.clone(),
            title,
            description,
            starts_at,
            ends_at,
            actions: actions.into_vec(),
        });

        self.total_upvotes(proposal_id).set(&payment_amount);
        self.upvotes(proposal_id).insert(proposer, payment_amount);

        proposal_id
    }

    #[payable("*")]
    #[endpoint(voteFor)]
    fn vote_for_endpoint(&self, #[payment_amount] payment_amount: BigUint, proposal_id: usize) {
        self.require_payment_token_governance_token();
        self.require_valid_proposal_id(proposal_id);
        require!(self.get_proposal_status(proposal_id) == ProposalStatus::Active, "proposal not active");

        let voter = self.blockchain().get_caller();

        self.emit_vote_for_event(&voter, proposal_id, &payment_amount);

        self.total_upvotes(proposal_id).update(|current| *current += &payment_amount);
        self.upvotes(proposal_id)
            .entry(voter)
            .and_modify(|current| *current += &payment_amount)
            .or_insert(payment_amount);
    }

    #[payable("*")]
    #[endpoint(voteAgainst)]
    fn vote_against_endpoint(&self, #[payment_amount] payment_amount: BigUint, proposal_id: usize) {
        self.require_payment_token_governance_token();
        self.require_valid_proposal_id(proposal_id);
        require!(self.get_proposal_status(proposal_id) == ProposalStatus::Active, "proposal not active");

        let downvoter = self.blockchain().get_caller();
        self.emit_vote_against_event(&downvoter, proposal_id, &payment_amount);
        self.total_downvotes(proposal_id).update(|current| *current += &payment_amount);

        self.downvotes(proposal_id)
            .entry(downvoter)
            .and_modify(|current| *current += &payment_amount)
            .or_insert(payment_amount);
    }

    #[endpoint(execute)]
    fn execute_endpoint(&self, proposal_id: usize) {
        require!(self.get_proposal_status(proposal_id) == ProposalStatus::Succeeded, "not ready to execute");

        let proposal = self.proposals().get(proposal_id);

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

        self.clear_proposal(proposal_id);
        self.emit_proposal_executed_event(proposal_id);
    }

    #[view(getProposalStatus)]
    fn get_proposal_status(&self, proposal_id: usize) -> ProposalStatus {
        if !self.proposal_exists(proposal_id) {
            return ProposalStatus::None;
        }

        let current_time = self.blockchain().get_block_timestamp();
        let proposal = self.proposals().get(proposal_id);

        if current_time >= proposal.starts_at && current_time < proposal.ends_at {
            return ProposalStatus::Active;
        }

        let total_for_votes = self.total_upvotes(proposal_id).get();
        let total_against_votes = self.total_downvotes(proposal_id).get();
        let quorum = self.quorum().get();
        let total_votes = &total_for_votes + &total_against_votes;
        let vote_for_percent = &total_for_votes / &total_votes;
        let vote_for_percent_to_pass = 66u64;

        if vote_for_percent > vote_for_percent_to_pass && total_for_votes >= quorum {
            ProposalStatus::Succeeded
        } else {
            ProposalStatus::Defeated
        }
    }

    #[view(getProposal)]
    fn get_proposal_view(&self, proposal_id: usize) -> OptionalValue<MultiValue5<ManagedBuffer, ManagedBuffer, ManagedAddress, u64, u64>> {
        if !self.proposal_exists(proposal_id) {
            OptionalValue::None
        } else {
            let proposal = self.proposals().get(proposal_id);
            OptionalValue::Some(
                (
                    proposal.title,
                    proposal.description,
                    proposal.proposer,
                    proposal.starts_at,
                    proposal.ends_at,
                )
                    .into(),
            )
        }
    }

    #[view(getProposalVotes)]
    fn get_proposal_votes_view(&self, proposal_id: usize) -> MultiValue2<BigUint, BigUint> {
        let upvotes = self.total_upvotes(proposal_id).get();
        let downvotes = self.total_downvotes(proposal_id).get();

        (upvotes, downvotes).into()
    }

    #[view(getProposalAddressVotes)]
    fn get_proposal_address_votes_view(&self, proposal_id: usize, address: ManagedAddress) -> MultiValue2<BigUint, BigUint> {
        let upvotes = self.upvotes(proposal_id).get(&address).unwrap_or_default();
        let downvotes = self.downvotes(proposal_id).get(&address).unwrap_or_default();

        (upvotes, downvotes).into()
    }

    // #[view(getProposalActions)]
    // fn get_proposal_actions_view(&self, proposal_id: usize) -> MultiValueVec<ActionAsMultiArg<Self::Api>> {
    //     if !self.proposal_exists(proposal_id) {
    //         return MultiValueVec::new();
    //     }

    //     let actions = self.proposals().get(proposal_id).actions;
    //     let mut actions_as_multiarg = Vec::with_capacity(actions.len());

    //     for action in actions.iter() {
    //         actions_as_multiarg.push(action.into_multiarg());
    //     }

    //     actions_as_multiarg.into()
    // }

    fn require_payment_token_governance_token(&self) {
        require!(self.call_value().token() == self.governance_token_id().get(), "invalid token");
    }

    fn require_valid_proposal_id(&self, proposal_id: usize) {
        require!(self.is_valid_proposal_id(proposal_id), "invalid proposal id");
    }

    fn is_valid_proposal_id(&self, proposal_id: usize) -> bool {
        proposal_id >= 1 && proposal_id <= self.proposals().len()
    }

    fn proposal_exists(&self, proposal_id: usize) -> bool {
        self.is_valid_proposal_id(proposal_id) && !self.proposals().item_is_empty(proposal_id)
    }

    /// specific votes/downvotes are not cleared,
    /// as they're used for reclaim tokens logic and cleared one by one
    fn clear_proposal(&self, proposal_id: usize) {
        self.proposals().clear_entry(proposal_id);
        self.total_upvotes(proposal_id).clear();
        self.total_downvotes(proposal_id).clear();
    }
}
