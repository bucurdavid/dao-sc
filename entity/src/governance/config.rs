use super::types::Proposal;

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait GovConfigModule {
    fn init_governance_module(&self, gov_token_id: &TokenIdentifier, vote_nft_token_id: &TokenIdentifier, initial_tokens: &BigUint) {
        require!(gov_token_id.is_valid_esdt_identifier(), "invalid edst");

        let initial_quorum = initial_tokens / &BigUint::from(20u64); // 5% of initial tokens
        let initial_min_tokens_for_proposing = initial_tokens / &BigUint::from(1000u64); // 0.1% of initial tokens
        let initial_voting_period_minutes = 4320u32; // 3 days

        self.governance_token_id().set_if_empty(&gov_token_id);
        self.vote_nft_token_id().set_if_empty(&vote_nft_token_id);

        self.try_change_quorum(BigUint::from(initial_quorum));
        self.try_change_min_proposal_vote_weight(BigUint::from(initial_min_tokens_for_proposing));
        self.try_change_voting_period_in_minutes(initial_voting_period_minutes);
    }

    #[endpoint(changeQuorum)]
    fn change_quorum(&self, new_value: BigUint) {
        self.require_caller_self();
        self.try_change_quorum(new_value);
    }

    #[endpoint(changeMinTokenBalanceForProposing)]
    fn change_min_proposal_vote_weight(&self, new_value: BigUint) {
        self.require_caller_self();
        self.try_change_min_proposal_vote_weight(new_value);
    }

    #[endpoint(changeVotingPeriodInMinutes)]
    fn change_voting_period_in_minutes(&self, new_value: u32) {
        self.require_caller_self();
        self.try_change_voting_period_in_minutes(new_value);
    }

    fn require_caller_self(&self) {
        let caller = self.blockchain().get_caller();
        let sc_address = self.blockchain().get_sc_address();

        require!(caller == sc_address, "action not allowed");
    }

    fn try_change_quorum(&self, quorum: BigUint) {
        require!(quorum != 0, "invalid quorum");
        self.quorum().set(&quorum);
    }

    fn try_change_min_proposal_vote_weight(&self, vote_weight: BigUint) {
        require!(vote_weight != 0, "min proposal vote weight can not be zero");
        self.min_proposal_vote_weight().set(&vote_weight);
    }

    fn try_change_voting_period_in_minutes(&self, voting_period: u32) {
        require!(voting_period != 0, "voting period (in minutes) can not be zero");
        self.voting_period_in_minutes().set(&voting_period);
    }

    #[storage_mapper("gov:proposals")]
    fn proposals(&self, id: u64) -> SingleValueMapper<Proposal<Self::Api>>;

    #[view(getProposalIdCounter)]
    #[storage_mapper("gov:proposals_id_counter")]
    fn proposal_id_counter(&self) -> SingleValueMapper<u64>;

    #[view(getGovTokenId)]
    #[storage_mapper("gov:governance_token_id")]
    fn governance_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getVoteNftTokenId)]
    #[storage_mapper("gov:vote_nft_token_id")]
    fn vote_nft_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getGovQuorum)]
    #[storage_mapper("gov:quorum")]
    fn quorum(&self) -> SingleValueMapper<BigUint>;

    #[view(getMinTokensForProposing)]
    #[storage_mapper("gov:min_proposal_vote_weight")]
    fn min_proposal_vote_weight(&self) -> SingleValueMapper<BigUint>;

    #[view(getVotingPeriodInMinutes)]
    #[storage_mapper("gov:voting_period_minutes")]
    fn voting_period_in_minutes(&self) -> SingleValueMapper<u32>;
}
