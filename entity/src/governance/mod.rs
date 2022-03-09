elrond_wasm::imports!();

pub mod configurable;
pub mod events;
pub mod storage;
pub mod types;

use types::*;

const MAX_BLOCK_GAS_LIMIT: u64 = 1_500_000_000;

#[elrond_wasm::module]
pub trait GovernanceModule: configurable::GovConfigurableModule + storage::GovStorageModule + events::GovEventsModule {
    #[payable("*")]
    #[endpoint(depositTokensForAction)]
    fn deposit_tokens_for_action_endpoint(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_nonce] payment_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
    ) {
        let caller = self.blockchain().get_caller();
        self.user_deposit_event(&caller, &payment_token, payment_nonce, &payment_amount);
    }

    #[endpoint(withdrawVoteTokens)]
    fn withdraw_gov_tokens_endpoint(&self, proposal_id: usize) {
        self.require_valid_proposal_id(proposal_id);
        require!(self.get_proposal_status(proposal_id) == ProposalStatus::None, "proposal is still active");

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
        #[var_args] actions: MultiValueVec<ActionAsMultiArg<Self::Api>>,
    ) -> usize {
        self.require_payment_token_governance_token();
        require!(payment_amount >= self.min_token_balance_for_proposing().get(), "not enough tokens");

        let mut gov_actions = ManagedVec::new();
        for action in actions.into_vec() {
            let (gas_limit, dest_address, token_id, token_nonce, amount, function_name, arguments) = action.into_tuple();
            let gov_action = Action {
                gas_limit,
                dest_address,
                token_id,
                token_nonce,
                amount,
                function_name,
                arguments,
            };

            gov_actions.push(gov_action);
        }

        require!(self.total_gas_needed(&gov_actions) < MAX_BLOCK_GAS_LIMIT, "actions require too much gas");

        let proposer = self.blockchain().get_caller();
        let current_block = self.blockchain().get_block_nonce();
        let proposal_id = self.proposals().len() + 1;

        self.proposal_created_event(proposal_id, &proposer, current_block, &title, &description, &gov_actions);

        let proposal = Proposal {
            proposer: proposer.clone(),
            title,
            description,
            actions: gov_actions,
        };
        let _ = self.proposals().push(&proposal);

        self.proposal_start_block(proposal_id).set(&current_block);
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

        self.vote_cast_event(&voter, proposal_id, &payment_amount);

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
        self.downvote_cast_event(&downvoter, proposal_id, &payment_amount);
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
        let total_gas_needed = self.total_gas_needed(&proposal.actions);
        let gas_left = self.blockchain().get_gas_left();

        require!(gas_left > total_gas_needed, "not enough gas to execute");

        for action in proposal.actions.iter() {
            let mut contract_call = self
                .send()
                .contract_call::<()>(action.dest_address, action.function_name)
                .with_gas_limit(action.gas_limit);

            if action.amount > 0 {
                contract_call = contract_call.add_token_transfer(action.token_id, action.token_nonce, action.amount);
            }

            for arg in action.arguments.iter() {
                contract_call.push_argument_raw_bytes(arg.to_boxed_bytes().as_slice());
            }

            contract_call.transfer_execute();
        }

        self.clear_proposal(proposal_id);
        self.proposal_executed_event(proposal_id);
    }

    #[view(getProposalStatus)]
    fn get_proposal_status(&self, proposal_id: usize) -> ProposalStatus {
        if !self.proposal_exists(proposal_id) {
            return ProposalStatus::None;
        }

        let current_block = self.blockchain().get_block_nonce();
        let voting_start = self.proposal_start_block(proposal_id).get();
        let voting_period = self.voting_period_in_blocks().get();
        let voting_end = voting_start + voting_period;

        if current_block >= voting_start && current_block < voting_end {
            return ProposalStatus::Active;
        }

        let total_for_votes = self.total_upvotes(proposal_id).get();
        let total_against_votes = self.total_downvotes(proposal_id).get();
        let quorum = self.quorum().get();

        if total_for_votes > total_against_votes && total_for_votes - total_against_votes >= quorum {
            ProposalStatus::Succeeded
        } else {
            ProposalStatus::Defeated
        }
    }

    #[view(getProposal)]
    fn get_proposal_view(&self, proposal_id: usize) -> OptionalValue<MultiValue3<ManagedBuffer, ManagedBuffer, ManagedAddress>> {
        if !self.proposal_exists(proposal_id) {
            OptionalValue::None
        } else {
            let proposal = self.proposals().get(proposal_id);
            OptionalValue::Some((proposal.title, proposal.description, proposal.proposer).into())
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

    fn total_gas_needed(&self, actions: &ManagedVec<Action<Self::Api>>) -> u64 {
        let mut total = 0;
        for action in actions {
            total += action.gas_limit;
        }

        total
    }

    /// specific votes/downvotes are not cleared,
    /// as they're used for reclaim tokens logic and cleared one by one
    fn clear_proposal(&self, proposal_id: usize) {
        self.proposals().clear_entry(proposal_id);
        self.proposal_start_block(proposal_id).clear();
        self.total_upvotes(proposal_id).clear();
        self.total_downvotes(proposal_id).clear();
    }
}
