use crate::governance::proposal::Proposal;

elrond_wasm::imports!();

pub const SEALED_NOT_SET: u8 = 0;
pub const SEALED_ON: u8 = 1;

#[elrond_wasm::module]
pub trait ConfigModule {
    fn require_caller_self(&self) {
        let caller = self.blockchain().get_caller();
        let sc_address = self.blockchain().get_sc_address();
        require!(caller == sc_address, "action not allowed by user");
    }

    fn require_caller_self_or_unsealed(&self) {
        if self.is_sealed() {
            self.require_caller_self();
        }
    }

    fn require_not_sealed(&self) {
        require!(!self.is_sealed(), "entity is sealed");
    }

    fn require_sealed(&self) {
        require!(self.is_sealed(), "entity is not sealed yet");
    }

    fn is_sealed(&self) -> bool {
        self.sealed().get() == SEALED_ON
    }

    fn require_payment_token_governance_token(&self) {
        require!(self.call_value().token() == self.governance_token_id().get(), "invalid token");
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

    #[view(getSealed)]
    #[storage_mapper("sealed")]
    fn sealed(&self) -> SingleValueMapper<u8>;

    #[view(getTokenId)]
    #[storage_mapper("token")]
    fn token(&self) -> FungibleTokenMapper<Self::Api>;

    #[view(getGovTokenId)]
    #[storage_mapper("governance_token_id")]
    fn governance_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getVoteNftTokenId)]
    #[storage_mapper("vote_nft_token")]
    fn vote_nft_token(&self) -> NonFungibleTokenMapper<Self::Api>;

    #[storage_mapper("proposals")]
    fn proposals(&self, id: u64) -> SingleValueMapper<Proposal<Self::Api>>;

    #[view(getProposalIdCounter)]
    #[storage_mapper("proposals_id_counter")]
    fn proposal_id_counter(&self) -> SingleValueMapper<u64>;

    #[view(getQuorum)]
    #[storage_mapper("quorum")]
    fn quorum(&self) -> SingleValueMapper<BigUint>;

    #[view(getMinProposalVoteWeight)]
    #[storage_mapper("min_proposal_vote_weight")]
    fn min_proposal_vote_weight(&self) -> SingleValueMapper<BigUint>;

    #[view(getVotingPeriodMinutes)]
    #[storage_mapper("voting_period_minutes")]
    fn voting_period_in_minutes(&self) -> SingleValueMapper<u32>;
}
