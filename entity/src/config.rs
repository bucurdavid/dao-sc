use multiversx_sc::api::ED25519_SIGNATURE_BYTE_LEN;

use crate::governance::proposal::Proposal;

multiversx_sc::imports!();

pub const VOTING_PERIOD_MINUTES_DEFAULT: usize = 4320; // 3 days
pub const VOTING_PERIOD_MINUTES_MAX: usize = 20_160; // 14 days
pub const MIN_PROPOSAL_VOTE_WEIGHT_DEFAULT: u64 = 1;
pub const QUORUM_DEFAULT: u64 = 1;

pub const POLL_MAX_OPTIONS: u8 = 20;

pub const GAS_LIMIT_SET_TOKEN_ROLES: u64 = 60_000_000;

pub const TOKEN_MAX_DECIMALS: u8 = 18;

pub type UserId = usize;

#[multiversx_sc::module]
pub trait ConfigModule {
    fn require_caller_self(&self) {
        let caller = self.blockchain().get_caller();
        let sc_address = self.blockchain().get_sc_address();
        require!(caller == sc_address, "action not allowed by user");
    }

    fn require_caller_trusted_host(&self) {
        let caller = self.blockchain().get_caller();
        let trusted_host_address = self.trusted_host_address().get();
        require!(caller == trusted_host_address, "action not allowed by user");
    }

    fn require_gov_token_set(&self) {
        require!(!self.gov_token_id().is_empty(), "gov token must be set");
    }

    fn require_payments_with_gov_token(&self) {
        let payments = self.call_value().all_esdt_transfers();
        let gov_token_id = self.gov_token_id().get();

        for payment in payments.into_iter() {
            require!(payment.token_identifier == gov_token_id, "invalid payment token");
        }
    }

    fn require_gov_tokens_available(&self, amount: &BigUint, nonce: u64) {
        let gov_token_id = self.gov_token_id().get();
        let protected = self.guarded_vote_tokens(&gov_token_id, nonce).get();
        let balance = self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(gov_token_id), nonce);
        let available = balance - protected;

        require!(amount <= &available, "not enough governance tokens available");
    }

    fn require_signed_by_trusted_host(&self, signable: &ManagedBuffer, signature: &ManagedByteArray<Self::Api, ED25519_SIGNATURE_BYTE_LEN>) {
        if self.trusted_host_address().is_empty() {
            return;
        }

        let trusted_host = self.trusted_host_address().get();
        let signable_hashed = self.crypto().keccak256(signable);

        // The error comes straight form the VM, the message is "invalid signature".
        self.crypto()
            .verify_ed25519(trusted_host.as_managed_buffer(), signable_hashed.as_managed_buffer(), &signature.as_managed_buffer());
    }

    fn try_change_governance_token(&self, token_id: &TokenIdentifier) {
        require!(token_id.is_valid_esdt_identifier(), "invalid token id");
        self.gov_token_id().set(token_id);
    }

    fn try_change_quorum(&self, quorum: BigUint) {
        require!(quorum != 0, "invalid quorum");
        self.quorum().set(&quorum);
    }

    fn try_change_min_vote_weight(&self, vote_weight: BigUint) {
        require!(vote_weight != 0, "min vote weight can not be zero");
        self.min_vote_weight().set(&vote_weight);
    }

    fn try_change_min_propose_weight(&self, vote_weight: BigUint) {
        require!(vote_weight != 0, "min propose weight can not be zero");
        self.min_propose_weight().set(&vote_weight);
    }

    fn try_change_voting_period_in_minutes(&self, voting_period: usize) {
        require!(voting_period != 0, "voting period can not be zero");
        require!(voting_period <= VOTING_PERIOD_MINUTES_MAX, "max voting period exceeded");
        self.voting_period_in_minutes().set(&voting_period);
    }

    #[storage_mapper("users")]
    fn users(&self) -> UserMapper;

    #[view(getTrustedHostAddress)]
    #[storage_mapper("trusted_host_addr")]
    fn trusted_host_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getGovTokenId)]
    #[storage_mapper("gov_token_id")]
    fn gov_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getGuardedVoteTokens)]
    #[storage_mapper("guarded_vote_tokens")]
    fn guarded_vote_tokens(&self, token_id: &TokenIdentifier, nonce: u64) -> SingleValueMapper<BigUint>;

    #[view(isLockingVoteTokens)]
    #[storage_mapper("lock_vote_tokens")]
    fn lock_vote_tokens(&self, token_id: &TokenIdentifier) -> SingleValueMapper<bool>;

    #[storage_mapper("proposals")]
    fn proposals(&self, id: u64) -> SingleValueMapper<Proposal<Self::Api>>;

    #[view(getProposalIdCounter)]
    #[storage_mapper("proposals_id_counter")]
    fn next_proposal_id(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("proposal_signers")]
    fn proposal_signers(&self, proposal_id: u64, role_name: &ManagedBuffer) -> UnorderedSetMapper<usize>;

    #[view(getProposalNftVotes)]
    #[storage_mapper("proposal_nft_votes")]
    fn proposal_nft_votes(&self, proposal_id: u64) -> UnorderedSetMapper<u64>;

    #[storage_mapper("proposal_poll")]
    fn proposal_poll(&self, proposal_id: u64, option_id: u8) -> SingleValueMapper<BigUint>;

    #[view(getWithdrawableProposalIds)]
    #[storage_mapper("withdrawable_proposal_ids")]
    fn withdrawable_proposal_ids(&self, voter: &ManagedAddress) -> UnorderedSetMapper<u64>;

    #[view(getWithdrawableVotes)]
    #[storage_mapper("withdrawable_votes")]
    fn withdrawable_votes(&self, proposal_id: u64, voter: &ManagedAddress) -> VecMapper<EsdtTokenPayment>;

    // keep for backwards compatibility
    #[view(getProposalAddressVotes)]
    #[storage_mapper("votes")]
    fn votes(&self, proposal_id: u64, voter: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[storage_mapper("known_th_proposals_ids")]
    fn known_trusted_host_proposal_ids(&self) -> UnorderedSetMapper<ManagedBuffer<Self::Api>>;

    #[view(getQuorum)]
    #[storage_mapper("quorum")]
    fn quorum(&self) -> SingleValueMapper<BigUint>;

    #[view(getMinVoteWeight)]
    #[storage_mapper("min_vote_weight")]
    fn min_vote_weight(&self) -> SingleValueMapper<BigUint>;

    #[view(getMinProposeWeight)]
    #[storage_mapper("min_proposal_vote_weight")]
    fn min_propose_weight(&self) -> SingleValueMapper<BigUint>;

    #[view(getVotingPeriodMinutes)]
    #[storage_mapper("voting_period_minutes")]
    fn voting_period_in_minutes(&self) -> SingleValueMapper<usize>;
}
