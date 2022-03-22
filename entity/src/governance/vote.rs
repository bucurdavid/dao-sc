elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use super::events;
use super::proposal;
use super::proposal::ProposalStatus;
use crate::config;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug, Clone)]
pub enum VoteType {
    For = 1,
    Against = 2,
}

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Debug)]
pub struct VoteNFTAttributes<M: ManagedTypeApi> {
    pub proposal_id: u64,
    pub vote_type: VoteType,
    pub vote_weight: BigUint<M>,
    pub voter: ManagedAddress<M>,
    pub payment: EsdtTokenPayment<M>,
}

#[elrond_wasm::module]
pub trait VoteModule: config::ConfigModule + proposal::ProposalModule + events::GovEventsModule {
    fn vote(&self, proposal_id: u64, vote_type: VoteType) {
        self.require_sealed();
        self.require_payment_token_governance_token();

        let voter = self.blockchain().get_caller();
        let payment = self.call_value().payment();
        let vote_weight = payment.amount.clone();
        let mut proposal = self.proposals(proposal_id).get();

        require!(self.get_proposal_status(&proposal) == ProposalStatus::Active, "proposal is not active");
        require!(vote_weight != 0u64, "can not vote with zero");

        match vote_type {
            VoteType::For => proposal.votes_for += &vote_weight,
            VoteType::Against => proposal.votes_against += &vote_weight,
        }

        self.create_vote_nft_and_send(&voter, proposal_id, vote_type.clone(), vote_weight.clone(), payment.clone());
        self.proposals(proposal_id).set(&proposal);
        self.emit_vote_event(proposal, vote_type, payment, vote_weight);
    }

    fn create_vote_nft_and_send(
        &self,
        voter: &ManagedAddress,
        proposal_id: u64,
        vote_type: VoteType,
        vote_weight: BigUint,
        payment: EsdtTokenPayment<Self::Api>,
    ) {
        let big_one = BigUint::from(1u64);
        let vote_nft_token_id = self.vote_nft_token().get_token_id();
        let attr = VoteNFTAttributes {
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
            &attr,
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
}
