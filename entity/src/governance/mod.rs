use self::types::{Action, Proposal, ProposalStatus, VoteNFTAttributes, VoteType};

elrond_wasm::imports!();

pub mod config;
pub mod events;
pub mod types;

#[elrond_wasm::module]
pub trait GovernanceModule: config::GovConfigModule + events::GovEventsModule {
    // #[endpoint(withdrawVoteTokens)]
    // fn withdraw_gov_tokens_endpoint(&self, proposal_id: u64) {
    //     self.require_valid_proposal_id(proposal_id);
    //     require!(
    //         self.get_proposal_status(proposal_id) != ProposalStatus::Active,
    //         "proposal is still active"
    //     );

    //     let caller = self.blockchain().get_caller();
    //     let gov_token_id = self.governance_token_id().get();
    //     let nr_votes_tokens = self.upvotes(proposal_id).get(&caller).unwrap_or_default();
    //     let nr_downvotes_tokens = self.downvotes(proposal_id).get(&caller).unwrap_or_default();
    //     let total_tokens = nr_votes_tokens + nr_downvotes_tokens;

    //     if total_tokens > 0 {
    //         self.upvotes(proposal_id).remove(&caller);
    //         self.downvotes(proposal_id).remove(&caller);
    //         self.send().direct(&caller, &gov_token_id, 0, &total_tokens, &[]);
    //     }
    // }

    #[payable("*")]
    #[endpoint(propose)]
    fn propose_endpoint(
        &self,
        title: ManagedBuffer,
        description: ManagedBuffer,
        #[var_args] actions: MultiValueManagedVec<Action<Self::Api>>,
    ) -> u64 {
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

    // #[endpoint(execute)]
    // fn execute_endpoint(&self, proposal_id: u64) {
    //     require!(self.get_proposal_status(proposal_id) == ProposalStatus::Succeeded, "not ready to execute");

    //     let proposal = self.proposals(proposal_id).get();

    //     for action in proposal.actions.iter() {
    //         let mut call = self
    //             .send()
    //             .contract_call::<()>(action.address, action.endpoint)
    //             .with_gas_limit(action.gas_limit);

    //         if action.amount > 0 {
    //             call = if action.token_id == TokenIdentifier::egld() {
    //                 call.with_egld_transfer(action.amount)
    //             } else {
    //                 call.add_token_transfer(action.token_id, action.token_nonce, action.amount)
    //             }
    //         }

    //         call.transfer_execute()
    //     }

    //     self.proposals(proposal_id).clear();
    //     // self.emit_proposal_executed_event(proposal_id);
    // }

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

    fn create_vote_nft_and_send(
        &self,
        voter: &ManagedAddress,
        proposal_id: u64,
        vote_type: VoteType,
        vote_weight: BigUint,
        payment: EsdtTokenPayment<Self::Api>,
    ) {
        let big_one = BigUint::from(1u64);
        let vote_nft_token_id = self.vote_nft_token_id().get();
        let attributes = VoteNFTAttributes {
            proposal_id,
            vote_type,
            vote_weight,
            voter: voter.clone(),
            payment,
        };

        let nonce = self.send().esdt_nft_create(
            &vote_nft_token_id,
            &big_one,
            &ManagedBuffer::new(),
            &BigUint::zero(),
            &ManagedBuffer::new(),
            &attributes,
            &ManagedVec::new(),
        );

        self.send().direct(&voter, &vote_nft_token_id, nonce, &big_one, &[]);
    }

    fn get_vote_nft_attr(&self, payment: &EsdtTokenPayment<Self::Api>) -> VoteNFTAttributes<Self::Api> {
        self.blockchain()
            .get_esdt_token_data(&self.blockchain().get_sc_address(), &payment.token_identifier, payment.token_nonce)
            .decode_attributes()
    }

    fn burn_vote_nft(&self, payment: EsdtTokenPayment<Self::Api>) {
        self.send()
            .esdt_local_burn(&payment.token_identifier, payment.token_nonce, &payment.amount);
    }

    fn require_payment_token_governance_token(&self) {
        require!(self.call_value().token() == self.governance_token_id().get(), "invalid token");
    }

    fn proposal_exists(&self, proposal_id: u64) -> bool {
        !self.proposals(proposal_id).is_empty()
    }
}
