multiversx_sc::imports!();

use super::proposal::{Proposal, VoteType};

#[multiversx_sc::module]
pub trait GovEventsModule {
    fn emit_propose_event(&self, proposer: ManagedAddress, proposal: &Proposal<Self::Api>, weight: BigUint, poll_option: u8) {
        self.propose_event(proposer, proposal.id, weight, poll_option);
    }

    fn emit_vote_event(&self, voter: ManagedAddress, proposal: &Proposal<Self::Api>, vote_type: VoteType, weight: BigUint, poll_option: u8) {
        match vote_type {
            VoteType::For => {
                self.vote_for_event(voter, proposal.id, weight, poll_option);
            }
            VoteType::Against => {
                self.vote_against_event(voter, proposal.id, weight, poll_option);
            }
        }
    }

    fn emit_sign_event(&self, signer: ManagedAddress, proposal: &Proposal<Self::Api>, poll_option: u8) {
        self.sign_event(signer, proposal.id, poll_option);
    }

    fn emit_execute_event(&self, proposal: &Proposal<Self::Api>) {
        self.execute_event(self.blockchain().get_caller(), proposal.id);
    }

    fn emit_direct_execute_event(&self) {
        self.direct_execute_event(self.blockchain().get_caller());
    }

    fn emit_cancel_event(&self, proposal: &Proposal<Self::Api>) {
        self.cancel_event(self.blockchain().get_caller(), proposal.id);
    }

    fn emit_withdraw_event(&self, proposal: &Proposal<Self::Api>) {
        self.withdraw_event(self.blockchain().get_caller(), proposal.id);
    }

    #[event("propose")]
    fn propose_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal_id: u64, #[indexed] weight: BigUint, #[indexed] poll_option: u8);

    #[event("vote_for")]
    fn vote_for_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal: u64, #[indexed] weight: BigUint, #[indexed] poll_option: u8);

    #[event("vote_against")]
    fn vote_against_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal: u64, #[indexed] weight: BigUint, #[indexed] poll_option: u8);

    #[event("sign")]
    fn sign_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal: u64, #[indexed] poll_option: u8);

    #[event("execute")]
    fn execute_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal: u64);

    #[event("direct_execute")]
    fn direct_execute_event(&self, #[indexed] caller: ManagedAddress);

    #[event("cancel")]
    fn cancel_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal: u64);

    #[event("withdraw")]
    fn withdraw_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal: u64);
}
