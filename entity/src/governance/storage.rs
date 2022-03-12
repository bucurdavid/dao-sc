elrond_wasm::imports!();

use super::Proposal;

#[elrond_wasm::module]
pub trait GovStorageModule {
    #[storage_mapper("gov:proposals")]
    fn proposals(&self) -> VecMapper<Proposal<Self::Api>>;

    #[storage_mapper("gov:proposal_start_block")]
    fn proposal_start_block(&self, proposal_id: usize) -> SingleValueMapper<u64>;

    #[storage_mapper("gov:proposal_start_timestamp")]
    fn proposal_start_timestamp(&self, proposal_id: usize) -> SingleValueMapper<u64>;

    #[storage_mapper("gov:upvotes")]
    fn upvotes(&self, proposal_id: usize) -> MapMapper<ManagedAddress, BigUint>;

    #[storage_mapper("gov:downvotes")]
    fn downvotes(&self, proposal_id: usize) -> MapMapper<ManagedAddress, BigUint>;

    #[storage_mapper("gov:total_upvotes")]
    fn total_upvotes(&self, proposal_id: usize) -> SingleValueMapper<BigUint>;

    #[storage_mapper("gov:total_downvotes")]
    fn total_downvotes(&self, proposal_id: usize) -> SingleValueMapper<BigUint>;

    // configurable

    #[view(getGovTokenId)]
    #[storage_mapper("gov:governance_token_id")]
    fn governance_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getGovQuorum)]
    #[storage_mapper("gov:quorum")]
    fn quorum(&self) -> SingleValueMapper<BigUint>;

    #[view(getMinTokensForProposing)]
    #[storage_mapper("gov:min_token_balance_propose")]
    fn min_token_balance_for_proposing(&self) -> SingleValueMapper<BigUint>;

    #[view(getVotingPeriodInBlocks)]
    #[storage_mapper("gov:voting_period_blocks")]
    fn voting_period_in_blocks(&self) -> SingleValueMapper<u64>;
}
