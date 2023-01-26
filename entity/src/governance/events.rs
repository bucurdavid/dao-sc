multiversx_sc::imports!();

use super::proposal::Proposal;
use super::vote::VoteType;

#[multiversx_sc::module]
pub trait GovEventsModule {
    fn emit_propose_event(&self, proposal: &Proposal<Self::Api>, weight: BigUint) {
        self.propose_event(
            self.blockchain().get_caller(),
            proposal.id,
            weight,
            self.blockchain().get_block_timestamp(),
            self.blockchain().get_block_epoch(),
        );
    }

    fn emit_vote_event(&self, proposal: &Proposal<Self::Api>, vote_type: VoteType, weight: BigUint) {
        match vote_type {
            VoteType::For => {
                self.vote_for_event(
                    self.blockchain().get_caller(),
                    proposal.id,
                    weight,
                    self.blockchain().get_block_timestamp(),
                    self.blockchain().get_block_epoch(),
                );
            }
            VoteType::Against => {
                self.vote_against_event(
                    self.blockchain().get_caller(),
                    proposal.id,
                    weight,
                    self.blockchain().get_block_timestamp(),
                    self.blockchain().get_block_epoch(),
                );
            }
        }
    }

    fn emit_sign_event(&self, proposal: &Proposal<Self::Api>) {
        self.sign_event(
            self.blockchain().get_caller(),
            proposal.id,
            self.blockchain().get_block_timestamp(),
            self.blockchain().get_block_epoch(),
        );
    }

    fn emit_execute_event(&self, proposal: &Proposal<Self::Api>) {
        self.execute_event(
            self.blockchain().get_caller(),
            proposal.id,
            self.blockchain().get_block_timestamp(),
            self.blockchain().get_block_epoch(),
        );
    }

    fn emit_withdraw_event(&self, proposal: &Proposal<Self::Api>) {
        self.withdraw_event(
            self.blockchain().get_caller(),
            proposal.id,
            self.blockchain().get_block_timestamp(),
            self.blockchain().get_block_epoch(),
        );
    }

    #[event("propose")]
    fn propose_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal_id: u64, #[indexed] weight: BigUint, #[indexed] timestamp: u64, #[indexed] epoch: u64);

    #[event("vote_for")]
    fn vote_for_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal: u64, #[indexed] weight: BigUint, #[indexed] timestamp: u64, #[indexed] epoch: u64);

    #[event("vote_against")]
    fn vote_against_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] proposal: u64,
        #[indexed] weight: BigUint,
        #[indexed] timestamp: u64,
        #[indexed] epoch: u64,
    );

    #[event("sign")]
    fn sign_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal: u64, #[indexed] timestamp: u64, #[indexed] epoch: u64);

    #[event("execute")]
    fn execute_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal: u64, #[indexed] timestamp: u64, #[indexed] epoch: u64);

    #[event("withdraw")]
    fn withdraw_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal: u64, #[indexed] timestamp: u64, #[indexed] epoch: u64);
}
