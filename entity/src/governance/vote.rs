elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use super::events;
use super::proposal;
use super::proposal::Proposal;
use super::proposal::ProposalStatus;
use crate::config;
use crate::permission;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug, Clone)]
pub enum VoteType {
    For = 1,
    Against = 2,
}

#[elrond_wasm::module]
pub trait VoteModule: config::ConfigModule + permission::PermissionModule + proposal::ProposalModule + events::GovEventsModule {
    fn vote(&self, proposal_id: u64, vote_type: VoteType, weight: BigUint, option_id: u8) {
        self.require_payments_with_gov_token();
        require!(weight > 0, "vote weight must be greater than 0");

        let mut proposal = self.proposals(proposal_id).get();
        let min_vote_weight = self.min_vote_weight().get();

        require!(weight >= min_vote_weight, "not enought vote weight");
        require!(self.get_proposal_status(&proposal) == ProposalStatus::Active, "proposal is not active");

        match vote_type {
            VoteType::For => proposal.votes_for += &weight,
            VoteType::Against => proposal.votes_against += &weight,
        }

        self.proposals(proposal_id).set(&proposal);
        self.cast_poll_vote(proposal_id, option_id, weight.clone());
        self.emit_vote_event(&proposal, vote_type, weight);
    }

    fn sign(&self, proposal_id: u64, option_id: u8) {
        let proposal = self.proposals(proposal_id).get();
        require!(self.get_proposal_status(&proposal) == ProposalStatus::Active, "proposal is not active");

        let signer = self.blockchain().get_caller();

        self.sign_for_all_roles(&signer, &proposal);
        self.cast_poll_vote(proposal.id, option_id, BigUint::from(1u8));
        self.emit_sign_event(&proposal);
    }

    fn sign_for_all_roles(&self, signer: &ManagedAddress, proposal: &Proposal<Self::Api>) {
        let signer_id = self.users().get_or_create_user(&signer);
        let signer_roles = self.user_roles(signer_id);

        for role in signer_roles.iter() {
            self.proposal_signers(proposal.id, &role).insert(signer_id);
        }
    }

    fn cast_poll_vote(&self, proposal_id: u64, option_id: u8, weight: BigUint) {
        if option_id == 0 || weight == 0 {
            return;
        }

        self.proposal_poll(proposal_id, option_id).update(|current| *current += weight);
    }

    fn withdraw_tokens(&self, proposal_id: u64) -> SCResult<(), ()> {
        if self.proposals(proposal_id).is_empty() {
            return Ok(());
        }

        let proposal = self.proposals(proposal_id).get();
        let status = self.get_proposal_status(&proposal);

        if status == ProposalStatus::Active || status == ProposalStatus::Pending {
            return Err(());
        }

        let caller = self.blockchain().get_caller();
        let mut returnables: ManagedVec<EsdtTokenPayment> = ManagedVec::new();

        // keep for backwards compatibility
        let gov_token_id = self.gov_token_id().get();
        let votes_mapper = self.votes(proposal_id, &caller);
        let votes = votes_mapper.get();

        if votes > 0 {
            self.guarded_vote_tokens(&gov_token_id, 0).update(|current| *current -= &votes);
            votes_mapper.clear();
            returnables.push(EsdtTokenPayment::new(gov_token_id, 0, votes));
        }
        // *end backwards compatibility

        for vote in self.withdrawable_votes(proposal_id, &caller).iter() {
            self.guarded_vote_tokens(&vote.token_identifier, vote.token_nonce)
                .update(|current| *current -= &vote.amount);

            returnables.push(vote);
        }

        self.withdrawable_votes(proposal_id, &caller).clear();
        self.emit_withdraw_event(&proposal);

        if !returnables.is_empty() {
            self.send().direct_multi(&caller, &returnables);
        }

        return Ok(());
    }
}
