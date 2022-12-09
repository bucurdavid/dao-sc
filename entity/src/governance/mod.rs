elrond_wasm::imports!();

use self::vote::VoteType;
use crate::config::{self, GAS_LIMIT_SET_TOKEN_ROLES, MIN_PROPOSAL_VOTE_WEIGHT_DEFAULT, POLL_MAX_OPTIONS, QUORUM_DEFAULT, VOTING_PERIOD_MINUTES_DEFAULT};
use crate::permission::{self, ROLE_BUILTIN_LEADER};
use errors::ALREADY_VOTED_WITH_TOKEN;
use proposal::{Action, ProposalStatus};

pub mod errors;
pub mod events;
pub mod proposal;
pub mod token;
pub mod vote;

#[elrond_wasm::module]
pub trait GovernanceModule:
    config::ConfigModule + permission::PermissionModule + events::GovEventsModule + proposal::ProposalModule + vote::VoteModule + token::TokenModule
{
    fn init_governance_module(&self) {
        self.next_proposal_id().set_if_empty(1);
        self.voting_period_in_minutes().set_if_empty(VOTING_PERIOD_MINUTES_DEFAULT);
        self.min_propose_weight().set_if_empty(BigUint::from(MIN_PROPOSAL_VOTE_WEIGHT_DEFAULT));
        self.quorum().set_if_empty(BigUint::from(QUORUM_DEFAULT));
    }

    fn configure_governance_token(&self, gov_token_id: TokenIdentifier, supply: BigUint, lock_vote_tokens: bool) {
        self.try_change_governance_token(&gov_token_id);
        self.lock_vote_tokens(&gov_token_id).set(lock_vote_tokens);

        if supply == 0 {
            return;
        }

        let initial_quorum = if &supply > &BigUint::from(100u64) {
            &supply * &BigUint::from(5u64) / &BigUint::from(100u64) // 5% of supply
        } else {
            BigUint::from(1u64)
        };

        let initial_min_tokens_for_proposing = if &supply > &BigUint::from(100u64) {
            &supply / &BigUint::from(100u64) // 1% of supply
        } else {
            BigUint::from(1u64)
        };

        self.try_change_quorum(BigUint::from(initial_quorum));
        self.try_change_min_propose_weight(BigUint::from(initial_min_tokens_for_proposing));
    }

    /// Initially configures the governance token if non is set already.
    /// It automatically calculates other governance setting defaults like quorum and minimum weight to propose.
    /// Can only be called by caller with leader role.
    #[endpoint(initGovToken)]
    fn init_gov_token_endpoint(&self, token_id: TokenIdentifier, supply: BigUint, lock_vote_tokens: bool) {
        require!(self.gov_token_id().is_empty(), "gov token is already set");
        self.require_caller_has_leader_role();

        self.configure_governance_token(token_id, supply, lock_vote_tokens);
    }

    /// Change the governance token.
    /// Automatically calculates other governance setting defaults like quorum and minimum weight to propose.
    /// Can only be called by the contract itself.
    #[endpoint(changeGovToken)]
    fn change_gov_token_endpoint(&self, token_id: TokenIdentifier, supply: BigUint, lock_vote_tokens: bool) {
        self.require_caller_self();
        self.configure_governance_token(token_id, supply, lock_vote_tokens);
    }

    /// Change the governance default quorum.
    /// Can only be called by the contract itself.
    #[endpoint(changeQuorum)]
    fn change_quorum_endpoint(&self, value: BigUint) {
        self.require_caller_self();
        self.require_gov_token_set();
        self.try_change_quorum(value);
    }

    /// Change the minimum weight required to vote.
    /// Can only be called by the contract itself.
    #[endpoint(changeMinVoteWeight)]
    fn change_min_vote_weight_endpoint(&self, value: BigUint) {
        self.require_caller_self();
        self.require_gov_token_set();
        self.try_change_min_vote_weight(value);
    }

    /// Change the minimumm weight required to create a proposal.
    /// Can only be called by the contract itself.
    #[endpoint(changeMinProposeWeight)]
    fn change_min_propose_weight_endpoint(&self, value: BigUint) {
        self.require_caller_self();
        self.require_gov_token_set();
        self.try_change_min_propose_weight(value);
    }

    /// Change the default voting period.
    /// Can only be called by the contract itself.
    /// Arguments:
    ///     - value: voting period duration **in minutes**
    #[endpoint(changeVotingPeriodMinutes)]
    fn change_voting_period_in_minutes_endpoint(&self, value: usize) {
        self.require_caller_self();
        self.try_change_voting_period_in_minutes(value);
    }

    /// Create a proposal with optional actions
    /// Arguments:
    ///     - trusted_host_id: a unique id given by the trusted host
    ///     - content_hash: the hash of the proposed content to verify integrity on the frontend
    ///     - content_sig: signature provided by the trusted host
    ///     - actions_hash: the hash of serialized actions to verify on execution. leave empty if no actions attached
    ///     - option_id: unique id of poll option. 0 = None
    ///     - permissions (optional): a list of permissions (their unique names) to be verified on proposal execution
    /// Payment:
    ///     - token id must be equal to configured governance token id
    ///     - amount must be greater than the min_propose_weight
    ///     - amount will be used to vote in favor (FOR) the proposal
    /// Returns an incremental proposal id
    #[payable("*")]
    #[endpoint(propose)]
    fn propose_endpoint(
        &self,
        trusted_host_id: ManagedBuffer,
        content_hash: ManagedBuffer,
        content_sig: ManagedBuffer,
        actions_hash: ManagedBuffer,
        option_id: u8,
        permissions: MultiValueManagedVec<ManagedBuffer>,
    ) -> u64 {
        let proposer = self.blockchain().get_caller();
        let permissions = permissions.into_vec();

        self.require_payments_with_gov_token();
        self.require_proposed_via_trusted_host(&trusted_host_id, &content_hash, content_sig, &actions_hash, &permissions);
        require!(!self.known_trusted_host_proposal_ids().contains(&trusted_host_id), "proposal already registered");

        let (allowed, policies) = self.can_propose(&proposer, &actions_hash, &permissions);
        require!(allowed, "action not allowed for user");

        let proposer_id = self.users().get_user_id(&proposer);
        let proposer_roles = self.user_roles(proposer_id);
        let vote_weight = self.get_weight_from_vote_payments();

        if proposer_roles.is_empty() || self.has_token_weighted_policy(&policies) {
            require!(vote_weight >= self.min_propose_weight().get(), "insufficient vote weight");
        }

        let proposal = self.create_proposal(content_hash, actions_hash, vote_weight.clone(), permissions, &policies);
        let proposal_id = proposal.id;

        if !proposer_roles.is_empty() {
            self.sign_for_all_roles(&proposer, &proposal);
        }

        self.commit_vote_payments(proposal_id);
        self.cast_poll_vote(proposal.id, option_id, vote_weight.clone());
        self.known_trusted_host_proposal_ids().insert(trusted_host_id);
        self.emit_propose_event(&proposal, vote_weight);

        proposal_id
    }

    /// Vote for of a proposal, optionally with a poll option.
    /// Payment:
    ///     - token id must be equal to configured governance token id
    ///     - amount must be greater than the min_vote_weight
    ///     - ESDTs will be deposited and locked until the voting period has ended
    ///     - NFTs/SFTs will be recorded as a vote and immediately returned
    #[payable("*")]
    #[endpoint(voteFor)]
    fn vote_for_endpoint(&self, proposal_id: u64, opt_option_id: OptionalValue<u8>) {
        let vote_weight = self.get_weight_from_vote_payments();
        let option_id = opt_option_id.into_option().unwrap_or_default();
        self.vote(proposal_id, VoteType::For, vote_weight, option_id);
        self.commit_vote_payments(proposal_id);
    }

    /// Vote against a proposal.
    /// Payment:
    ///     - token id must be equal to configured governance token id
    ///     - amount must be greater than the min_vote_weight
    ///     - ESDTs will be deposited and locked until the voting period has ended
    ///     - NFTs/SFTs will be recorded as a vote and immediately returned
    #[payable("*")]
    #[endpoint(voteAgainst)]
    fn vote_against_endpoint(&self, proposal_id: u64, opt_option_id: OptionalValue<u8>) {
        let vote_weight = self.get_weight_from_vote_payments();
        let option_id = opt_option_id.into_option().unwrap_or_default();
        self.vote(proposal_id, VoteType::Against, vote_weight, option_id);
        self.commit_vote_payments(proposal_id);
    }

    /// Sign a proposal, optionally with a poll option.
    /// This is often required by role members to approve actions protected by policies.
    #[endpoint(sign)]
    fn sign_endpoint(&self, proposal_id: u64, opt_option_id: OptionalValue<u8>) {
        let option_id = opt_option_id.into_option().unwrap_or_default();
        self.sign(proposal_id, option_id);
    }

    /// Execute the actions of a succeeded proposal.
    /// This will update the proposals status to 'executed'.
    #[endpoint(execute)]
    fn execute_endpoint(&self, proposal_id: u64, actions: MultiValueManagedVec<Action<Self::Api>>) {
        require!(!actions.is_empty(), "no actions to execute");
        require!(!self.proposals(proposal_id).is_empty(), "proposal not found");

        let actions = actions.into_vec();
        let actions_hash = self.calculate_actions_hash(&actions);
        let mut proposal = self.proposals(proposal_id).get();

        require!(proposal.actions_hash == actions_hash, "actions have been corrupted");
        require!(self.get_proposal_status(&proposal) == ProposalStatus::Succeeded, "proposal is not executable");

        let actual_permissions = self.get_actual_permissions(&proposal, &actions);
        require!(proposal.permissions == actual_permissions, "untruthful permissions announced");

        proposal.was_executed = true;
        self.proposals(proposal_id).set(&proposal);

        self.execute_actions(&actions);
        self.emit_execute_event(&proposal);
    }

    /// Withdraw ESDT governance tokens once the proposals voting period has ended.
    /// Used by members who voted FOR or AGAINST a proposal using ESDTs.
    #[endpoint(withdraw)]
    fn withdraw_endpoint(&self) {
        let caller = self.blockchain().get_caller();

        for proposal_id in self.withdrawable_proposal_ids(&caller).iter() {
            self.withdraw_tokens(proposal_id);
        }
    }

    /// Issue and configure a fresh governance ESDT owned by the smart contract.
    /// It automatically calculates other governance setting defaults like quorum and minimum weight to propose.
    /// The initially minted tokens (supply) will be send to the caller.
    /// Can only be called by caller with leader role.
    /// Payment: EGLD in amount required by the protocol.
    #[payable("EGLD")]
    #[endpoint(issueGovToken)]
    fn issue_gov_token_endpoint(&self, token_name: ManagedBuffer, token_ticker: ManagedBuffer, supply: BigUint) {
        require!(self.gov_token_id().is_empty(), "governance token already set");

        let caller = self.blockchain().get_caller();
        let user_id = self.users().get_user_id(&caller);
        let is_leader = self.user_roles(user_id).contains(&ManagedBuffer::from(ROLE_BUILTIN_LEADER));

        require!(is_leader, "only allowed for leader");

        self.issue_gov_token(token_name, token_ticker, supply)
            .with_callback(self.callbacks().gov_token_issue_callback(&caller))
            .call_and_exit();
    }

    /// Set local Mint & Burn roles of the governance token for the smart contract.
    /// Usually called after `issueGovToken`.
    #[endpoint(setGovTokenLocalRoles)]
    fn set_gov_token_local_roles_endpoint(&self) {
        require!(!self.gov_token_id().is_empty(), "gov token must be set");

        let gov_token_id = self.gov_token_id().get();
        let entity_address = self.blockchain().get_sc_address();
        let roles = [EsdtLocalRole::Mint, EsdtLocalRole::Burn];

        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(&entity_address, &gov_token_id, (&roles[..]).into_iter().cloned())
            .with_gas_limit(GAS_LIMIT_SET_TOKEN_ROLES)
            .async_call()
            .call_and_exit();
    }

    #[payable("*")]
    #[callback]
    fn gov_token_issue_callback(&self, initial_caller: &ManagedAddress, #[call_result] result: ManagedAsyncCallResult<()>) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {
                let payment = self.call_value().single_esdt();
                self.send().direct_esdt(&initial_caller, &payment.token_identifier, 0, &payment.amount);
                self.configure_governance_token(payment.token_identifier, payment.amount, true);
            }
            ManagedAsyncCallResult::Err(_) => self.send_received_egld(&initial_caller),
        }
    }

    /// Mint tokens of any ESDT locally.
    /// This call will fail if the smart contract does not have the `ESDTRoleLocalMint` for the provided token id.
    #[endpoint(mint)]
    fn mint_endpoint(&self, token: TokenIdentifier, nonce: u64, amount: BigUint) {
        self.require_caller_self();
        self.send().esdt_local_mint(&token, nonce, &amount);
    }

    /// Burn tokens of any ESDT locally.
    /// This call will fail if the smart contract does not have the `ESDTRoleLocalBurn` for the provided token id.
    #[endpoint(burn)]
    fn burn_endpoint(&self, token: TokenIdentifier, nonce: u64, amount: BigUint) {
        self.require_caller_self();
        self.send().esdt_local_burn(&token, nonce, &amount);
    }

    #[view(getProposal)]
    fn get_proposal_view(&self, proposal_id: u64) -> OptionalValue<MultiValue6<ManagedBuffer, ManagedBuffer, ManagedAddress, u64, u64, bool>> {
        if !self.proposal_exists(proposal_id) {
            OptionalValue::None
        } else {
            let proposal = self.proposals(proposal_id).get();
            OptionalValue::Some(
                (
                    proposal.content_hash,
                    proposal.actions_hash,
                    proposal.proposer,
                    proposal.starts_at,
                    proposal.ends_at,
                    proposal.was_executed,
                )
                    .into(),
            )
        }
    }

    #[view(getProposalStatus)]
    fn get_proposal_status_view(&self, proposal_id: u64) -> ProposalStatus {
        require!(!self.proposals(proposal_id).is_empty(), "proposal not found");

        self.get_proposal_status(&self.proposals(proposal_id).get())
    }

    #[view(getProposalVotes)]
    fn get_proposal_votes_view(&self, proposal_id: u64) -> MultiValue2<BigUint, BigUint> {
        let proposal = self.proposals(proposal_id).get();

        (proposal.votes_for, proposal.votes_against).into()
    }

    #[view(getProposalSigners)]
    fn get_proposal_signers_view(&self, proposal_id: u64) -> MultiValueEncoded<ManagedAddress> {
        let proposal = self.proposals(proposal_id).get();
        let proposer_id = self.users().get_user_id(&proposal.proposer);
        let proposer_roles = self.user_roles(proposer_id);
        let mut signers = MultiValueEncoded::new();

        for role in proposer_roles.iter() {
            for signer_id in self.proposal_signers(proposal.id, &role).iter() {
                let address = self.users().get_user_address_unchecked(signer_id);
                if !signers.to_vec().contains(&address) {
                    signers.push(address);
                }
            }
        }
        signers
    }

    #[view(getProposalSignatureRoleCounts)]
    fn get_proposal_signature_role_counts_view(&self, proposal_id: u64) -> MultiValueEncoded<MultiValue2<ManagedBuffer, usize>> {
        let proposal = self.proposals(proposal_id).get();
        let proposer_id = self.users().get_user_id(&proposal.proposer);
        let proposer_roles = self.user_roles(proposer_id);
        let mut signers = MultiValueEncoded::new();

        for role in proposer_roles.iter() {
            let signer_count = self.proposal_signers(proposal.id, &role).len();
            if signer_count > 0 {
                signers.push((role, signer_count).into());
            }
        }
        signers
    }

    #[view(getProposalPollResults)]
    fn get_proposal_poll_results_view(&self, proposal_id: u64) -> MultiValueEncoded<BigUint> {
        let mut results = MultiValueEncoded::new();

        for option_id in 1..=POLL_MAX_OPTIONS {
            results.push(self.proposal_poll(proposal_id, option_id).get());
        }

        results
    }

    fn get_weight_from_vote_payments(&self) -> BigUint {
        self.call_value()
            .all_esdt_transfers()
            .into_iter()
            .fold(BigUint::zero(), |carry, payment| carry + &payment.amount)
    }

    /// Processes received vote payment tokens.
    /// Either keeps track of them for withdrawals or sends them back immediately depending on the token type.
    /// - ESDTs will >always< be deposited/locked in the contract.
    /// - NFTs, SFTs & MetaESDTs are only locked if locked_vote_tokens is set to true (default).
    /// Fails if the NFT's nonce has been used to vote previously.
    fn commit_vote_payments(&self, proposal_id: u64) {
        let payments = self.call_value().all_esdt_transfers();
        let caller = self.blockchain().get_caller();
        let mut returnables = ManagedVec::new();

        for payment in payments.into_iter() {
            if payment.token_nonce == 0 || self.lock_vote_tokens(&payment.token_identifier).get() {
                self.withdrawable_proposal_ids(&caller).insert(proposal_id);
                self.withdrawable_proposal_token_nonces(proposal_id, &caller).insert(payment.token_nonce);
                self.guarded_vote_tokens(&payment.token_identifier, payment.token_nonce)
                    .update(|current| *current += &payment.amount);
                self.withdrawable_votes(proposal_id, &caller, &payment.token_identifier, payment.token_nonce)
                    .update(|current| *current += &payment.amount);
            } else {
                let inserted = self.proposal_nft_votes(proposal_id).insert(payment.token_nonce);
                require!(inserted, ALREADY_VOTED_WITH_TOKEN);
                returnables.push(payment);
            }
        }

        if !returnables.is_empty() {
            self.send().direct_multi(&caller, &returnables);
        }
    }
}
