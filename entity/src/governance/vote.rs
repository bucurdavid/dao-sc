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
    fn vote(&self, proposal_id: u64, vote_type: VoteType) {
        self.require_payment_token_governance_token();

        let voter = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
        let vote_weight = payment.amount.clone();
        let mut proposal = self.proposals(proposal_id).get();

        require!(vote_weight > 0, "not enough vote weight");
        require!(self.get_proposal_status(&proposal) == ProposalStatus::Active, "proposal is not active");

        match vote_type {
            VoteType::For => proposal.votes_for += &vote_weight,
            VoteType::Against => proposal.votes_against += &vote_weight,
        }

        self.proposals(proposal_id).set(&proposal);
        self.protected_vote_tokens(&payment.token_identifier).update(|current| *current += &payment.amount);
        self.votes(proposal_id, &voter).update(|current| *current += &payment.amount);
        self.withdrawable_proposal_ids(&voter).insert(proposal_id);
        self.emit_vote_event(proposal, vote_type, payment, vote_weight);
    }

    fn sign(&self, proposal_id: u64) {
        let proposal = self.proposals(proposal_id).get();
        require!(self.get_proposal_status(&proposal) == ProposalStatus::Active, "proposal is not active");

        let signer = self.blockchain().get_caller();
        self.sign_for_all_roles(&signer, &proposal);
        self.emit_sign_event(proposal);
    }

    fn sign_for_all_roles(&self, signer: &ManagedAddress, proposal: &Proposal<Self::Api>) {
        let signer_id = self.users().get_or_create_user(&signer);
        let signer_roles = self.user_roles(signer_id);

        for role in signer_roles.iter() {
            self.proposal_signers(proposal.id, &role).insert(signer_id);
        }
    }

    fn withdraw_tokens(&self, proposal_id: u64) {
        let caller = self.blockchain().get_caller();
        let proposal = self.proposals(proposal_id).get();
        let status = self.get_proposal_status(&proposal);

        if status == ProposalStatus::Active || status == ProposalStatus::Pending {
            return;
        }

        let gov_token_id = self.gov_token_id().get();
        let votes_mapper = self.votes(proposal_id, &caller);
        let votes = votes_mapper.get();

        if votes > 0 {
            votes_mapper.clear();
            self.protected_vote_tokens(&gov_token_id).update(|current| *current -= &votes);
            self.withdrawable_proposal_ids(&caller).swap_remove(&proposal_id);
            self.send().direct_esdt(&caller, &gov_token_id, 0, &votes);
        }
    }
}
