elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait GovEventsModule {
    #[event("proposalCreated")]
    fn emit_proposal_created_event(
        &self,
        #[indexed] proposal_id: usize,
        #[indexed] proposer: &ManagedAddress,
        #[indexed] starts_at: u64,
        #[indexed] ends_at: u64,
        #[indexed] title: &ManagedBuffer,
        #[indexed] description: &ManagedBuffer,
    );

    #[event("votedFor")]
    fn emit_vote_for_event(&self, #[indexed] voter: &ManagedAddress, #[indexed] proposal_id: usize, votes: &BigUint);

    #[event("votedAgainst")]
    fn emit_vote_against_event(&self, #[indexed] voter: &ManagedAddress, #[indexed] proposal_id: usize, votes: &BigUint);

    #[event("proposalExecuted")]
    fn emit_proposal_executed_event(&self, #[indexed] proposal_id: usize);
}
