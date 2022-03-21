elrond_wasm::imports!();

use crate::config;
use proposal::{Action, Proposal, ProposalStatus};
use vote::VoteType;

pub mod events;
pub mod proposal;
pub mod vote;

#[elrond_wasm::module]
pub trait GovernanceModule: config::ConfigModule + events::GovEventsModule + proposal::ProposalModule + vote::VoteModule {
    fn init_governance_module(&self, gov_token_id: &TokenIdentifier, initial_tokens: &BigUint) {
        require!(gov_token_id.is_valid_esdt_identifier(), "invalid edst");

        let initial_quorum = initial_tokens / &BigUint::from(20u64); // 5% of initial tokens
        let initial_min_tokens_for_proposing = initial_tokens / &BigUint::from(1000u64); // 0.1% of initial tokens
        let initial_voting_period_minutes = 4320u32; // 3 days

        self.governance_token_id().set_if_empty(&gov_token_id);
        self.try_change_quorum(BigUint::from(initial_quorum));
        self.try_change_min_proposal_vote_weight(BigUint::from(initial_min_tokens_for_proposing));
        self.try_change_voting_period_in_minutes(initial_voting_period_minutes);
    }

    #[endpoint(changeQuorum)]
    fn change_quorum(&self, new_value: BigUint) {
        self.require_caller_self_or_unsealed();
        self.try_change_quorum(new_value);
    }

    #[endpoint(changeMinProposalVoteWeight)]
    fn change_min_proposal_vote_weight(&self, new_value: BigUint) {
        self.require_caller_self_or_unsealed();
        self.try_change_min_proposal_vote_weight(new_value);
    }

    #[endpoint(changeVotingPeriodMinutes)]
    fn change_voting_period_in_minutes(&self, new_value: u32) {
        self.require_caller_self_or_unsealed();
        self.try_change_voting_period_in_minutes(new_value);
    }

    #[endpoint(redeem)]
    fn redeem_endpoint(&self) {
        let payment = self.call_value().payment();
        let caller = self.blockchain().get_caller();
        let vote_nft_id = self.vote_nft_token().get_token_id();
        let attributes = self.get_vote_nft_attr(&payment);
        let status = self.get_proposal_status(attributes.proposal_id);
        let proposal = self.proposals(attributes.proposal_id).get();

        require!(payment.token_identifier == vote_nft_id, "invalid vote position");
        require!(status != ProposalStatus::Active, "proposal is still active");

        self.send()
            .direct(&caller, &payment.token_identifier, payment.token_nonce, &payment.amount, &[]);

        self.burn_vote_nft(payment.clone());
        self.emit_redeem_event(proposal, payment, attributes);
    }

    #[payable("*")]
    #[endpoint(propose)]
    fn propose_endpoint(
        &self,
        title: ManagedBuffer,
        description: ManagedBuffer,
        #[var_args] actions: MultiValueManagedVec<Action<Self::Api>>,
    ) -> u64 {
        self.require_sealed();
        self.require_payment_token_governance_token();

        let payment = self.call_value().payment();
        let vote_weight = payment.amount.clone();
        let proposer = self.blockchain().get_caller();
        let proposal_id = self.proposal_id_counter().get();
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
            actions: actions.into_vec(),
            votes_for: vote_weight.clone(),
            votes_against: BigUint::zero(),
        };

        self.proposals(proposal_id.clone()).set(&proposal);
        self.proposal_id_counter().set(proposal.id + 1);
        self.create_vote_nft_and_send(&proposer, proposal.id, VoteType::For, vote_weight.clone(), payment.clone());
        self.emit_propose_event(proposal, payment, vote_weight);

        proposal_id
    }

    #[payable("*")]
    #[endpoint(voteFor)]
    fn vote_for_endpoint(&self, proposal_id: u64) {
        self.vote(proposal_id, VoteType::For)
    }

    #[payable("*")]
    #[endpoint(voteAgainst)]
    fn vote_against_endpoint(&self, proposal_id: u64) {
        self.vote(proposal_id, VoteType::Against)
    }

    fn vote(&self, proposal_id: u64, vote_type: VoteType) {
        self.require_sealed();
        self.require_payment_token_governance_token();
        require!(self.get_proposal_status(proposal_id) == ProposalStatus::Active, "proposal is not active");

        let voter = self.blockchain().get_caller();
        let payment = self.call_value().payment();
        let vote_weight = payment.amount.clone();
        let mut proposal = self.proposals(proposal_id).get();

        require!(vote_weight != 0u64, "can not vote with zero");

        match vote_type {
            VoteType::For => proposal.votes_for += &vote_weight,
            VoteType::Against => proposal.votes_against += &vote_weight,
        }

        self.create_vote_nft_and_send(&voter, proposal_id, vote_type.clone(), vote_weight.clone(), payment.clone());
        self.proposals(proposal_id).set(&proposal);
        self.emit_vote_event(proposal, vote_type, payment, vote_weight);
    }

    #[endpoint(execute)]
    fn execute_endpoint(&self, proposal_id: u64) {
        self.require_sealed();
        require!(self.get_proposal_status(proposal_id) == ProposalStatus::Succeeded, "not ready to execute");

        let proposal = self.proposals(proposal_id).get();

        self.execute_proposal(&proposal);

        self.emit_execute_event(proposal);
    }

    #[view(getProposalStatus)]
    fn get_proposal_status(&self, proposal_id: u64) -> ProposalStatus {
        if !self.proposal_exists(proposal_id) {
            return ProposalStatus::None;
        }

        let current_time = self.blockchain().get_block_timestamp();
        let proposal = self.proposals(proposal_id).get();

        if current_time >= proposal.starts_at && current_time < proposal.ends_at {
            return ProposalStatus::Active;
        }

        let quorum = self.quorum().get();
        let total_votes = &proposal.votes_for + &proposal.votes_against;
        let vote_for_percent = &proposal.votes_for / &total_votes;
        let vote_for_percent_to_pass = 66u64;

        if vote_for_percent > vote_for_percent_to_pass && &proposal.votes_for >= &quorum {
            ProposalStatus::Succeeded
        } else {
            ProposalStatus::Defeated
        }
    }

    #[view(getProposal)]
    fn get_proposal_view(&self, proposal_id: u64) -> OptionalValue<MultiValue5<ManagedBuffer, ManagedBuffer, ManagedAddress, u64, u64>> {
        if !self.proposal_exists(proposal_id) {
            OptionalValue::None
        } else {
            let proposal = self.proposals(proposal_id).get();
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
    fn get_proposal_votes_view(&self, proposal_id: u64) -> MultiValue2<BigUint, BigUint> {
        let proposal = self.proposals(proposal_id).get();

        (proposal.votes_for, proposal.votes_against).into()
    }

    // #[view(getProposalAddressVotes)]
    // fn get_proposal_address_votes_view(&self, proposal_id: u64, address: ManagedAddress) -> MultiValue2<BigUint, BigUint> {
    //     let upvotes = self.upvotes(proposal_id).get(&address).unwrap_or_default();
    //     let downvotes = self.downvotes(proposal_id).get(&address).unwrap_or_default();

    //     (upvotes, downvotes).into()
    // }

    // #[view(getProposalActions)]
    // fn get_proposal_actions_view(&self, proposal_id: u64) -> MultiValueVec<ActionAsMultiArg<Self::Api>> {
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

    fn proposal_exists(&self, proposal_id: u64) -> bool {
        !self.proposals(proposal_id).is_empty()
    }
}
