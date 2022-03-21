elrond_wasm::imports!();
elrond_wasm::derive_imports!();

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
pub trait VoteModule: config::ConfigModule {
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
