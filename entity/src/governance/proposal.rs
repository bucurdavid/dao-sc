elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::api::KECCAK256_RESULT_LEN;

use crate::config;
use crate::permission;
use crate::permission::PermissionDetails;
use crate::permission::{Policy, PolicyMethod, ROLE_BUILTIN_LEADER};
use core::convert::TryFrom;

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct Proposal<M: ManagedTypeApi> {
    pub id: u64,
    pub proposer: ManagedAddress<M>,
    pub content_hash: ManagedBuffer<M>,
    pub actions_hash: ManagedBuffer<M>,
    pub starts_at: u64,
    pub ends_at: u64,
    pub was_executed: bool,
    pub votes_for: BigUint<M>,
    pub votes_against: BigUint<M>,
    pub permissions: ManagedVec<M, ManagedBuffer<M>>,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem)]
pub struct Action<M: ManagedTypeApi> {
    pub destination: ManagedAddress<M>,
    pub endpoint: ManagedBuffer<M>,
    pub value: BigUint<M>,
    pub payments: ManagedVec<M, EsdtTokenPayment<M>>,
    pub arguments: ManagedVec<M, ManagedBuffer<M>>,
    pub gas_limit: u64,
}

pub type ActionAsMultiArg<M> = MultiValue6<
    ManagedAddress<M>,
    ManagedBuffer<M>,
    BigUint<M>,
    ManagedVec<M, EsdtTokenPayment<M>>,
    ManagedVec<M, ManagedBuffer<M>>,
    u64
>;

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Debug)]
pub enum ProposalStatus {
    Pending,
    Active,
    Defeated,
    Succeeded,
    Executed,
}

#[elrond_wasm::module]
pub trait ProposalModule: config::ConfigModule + permission::PermissionModule {
    fn create_proposal(
        &self,
        content_hash: ManagedBuffer,
        actions_hash: ManagedBuffer,
        vote_weight: BigUint,
        permissions: ManagedVec<ManagedBuffer>,
        policies: &ManagedVec<Policy<Self::Api>>,
    ) -> Proposal<Self::Api> {
        let proposer = self.blockchain().get_caller();
        let proposal_id = self.next_proposal_id().get();

        if !actions_hash.is_empty() {
            require!(actions_hash.len() == KECCAK256_RESULT_LEN, "invalid actions hash");
        }

        let voting_period_minutes = policies.iter()
            .map(|p| p.voting_period_minutes)
            .max()
            .unwrap_or_else(|| self.voting_period_in_minutes().get());

        let starts_at = self.blockchain().get_block_timestamp();
        let ends_at = starts_at + voting_period_minutes as u64 * 60;

        let proposal = Proposal {
            id: proposal_id,
            proposer: proposer.clone(),
            content_hash,
            starts_at,
            ends_at,
            was_executed: false,
            actions_hash,
            votes_for: vote_weight.clone(),
            votes_against: BigUint::zero(),
            permissions,
        };

        self.proposals(proposal_id).set(&proposal);
        self.votes(proposal_id, &proposer).set(vote_weight);
        self.withdrawable_proposal_ids(&proposer).insert(proposal_id);
        self.next_proposal_id().set(proposal_id + 1);

        proposal
    }

    fn get_proposal_status(&self, proposal: &Proposal<Self::Api>) -> ProposalStatus {
        let current_time = self.blockchain().get_block_timestamp();

        if proposal.was_executed {
            return ProposalStatus::Executed;
        }

        if current_time >= proposal.starts_at && current_time < proposal.ends_at {
            return ProposalStatus::Active;
        }

        if proposal.actions_hash.is_empty() || proposal.permissions.is_empty() {
            return match self.has_sufficient_votes(&proposal, &self.quorum().get()) {
                true => ProposalStatus::Succeeded,
                false => ProposalStatus::Defeated,
            };
        }

        let proposer_id = self.users().get_user_id(&proposal.proposer);
        let proposer_roles = self.user_roles(proposer_id);
        let mut fulfilled_all = true;

        for permission in proposal.permissions.into_iter() {
            let fulfilled_perm = proposer_roles.iter()
                .map(|role| if let Some(policy) = self.policies(&role).get(&permission) {
                    match policy.method {
                        PolicyMethod::Weight => self.has_sufficient_votes(&proposal, &policy.quorum),
                        PolicyMethod::One => self.proposal_signers(proposal.id, &role).contains(&proposer_id),
                        PolicyMethod::All => self.proposal_signers(proposal.id, &role).len() >= self.roles_member_amount(&role).get(),
                        PolicyMethod::Quorum => BigUint::from(self.proposal_signers(proposal.id, &role).len()) >= policy.quorum,
                    }
                } else {
                    self.proposal_signers(proposal.id, &role).len() > self.roles_member_amount(&role).get() / 2
                })
                .find(|fulfilled| !fulfilled)
                .is_none();

            if !fulfilled_perm {
                fulfilled_all = false;
            }
        }

        if fulfilled_all {
            return ProposalStatus::Succeeded;
        }

        ProposalStatus::Defeated
    }

    fn execute_actions(&self, actions: &ManagedVec<Action<Self::Api>>) {
        let gov_token_id = self.gov_token_id().get();

        for action in actions.iter() {
            let mut call = self
                .send()
                .contract_call::<()>(action.destination, action.endpoint)
                .with_arguments_raw(ManagedArgBuffer::from(action.arguments))
                .with_gas_limit(action.gas_limit);

            if action.value > 0 {
                call = call.with_egld_transfer(action.value);
                call.transfer_execute();
                break;
            }

            for payment in action.payments.iter() {
                if payment.token_identifier == gov_token_id {
                    self.require_governance_tokens_available(&payment.amount);
                }

                call = call.add_esdt_token_transfer(payment.token_identifier, payment.token_nonce, payment.amount);
            }

            call.transfer_execute()
        }
    }

    fn can_propose(&self, proposer: &ManagedAddress, actions_hash: &ManagedBuffer, permissions: &ManagedVec<ManagedBuffer>) -> (bool, ManagedVec<Policy<Self::Api>>) {
        let proposer_id = self.users().get_user_id(proposer);
        let proposer_roles = self.user_roles(proposer_id);
        let mut policies = ManagedVec::new();

        // no actions -> always allowed
        if actions_hash.is_empty() && permissions.is_empty() {
            return (true, policies);
        }

        // has roles -> always allowed
        if !proposer_roles.is_empty() {
            return (true, policies);
        }

        // if fellowship has leaders, only allow if has a role
        if self.does_leader_role_exist() && proposer_roles.is_empty() {
            return (false, policies);
        }

        let mut allowed = false;

        for role in proposer_roles.iter() {
            if role == ManagedBuffer::from(ROLE_BUILTIN_LEADER) {
                allowed = true;
            }

            for permission in permissions.into_iter() {
                let policy = self.policies(&role).get(&permission);

                if policy.is_some() {
                    policies.push(policy.unwrap());
                    allowed = true;
                }
            }
        }

        (allowed, policies)
    }

    fn calculate_actions_hash(&self, actions: &ManagedVec<Action<Self::Api>>) -> ManagedBuffer<Self::Api> {
        let mut serialized = ManagedBuffer::new();

        for action in actions.iter() {
            serialized.append(&sc_format!("{:x}{}{}", action.destination.as_managed_buffer(), action.endpoint, action.value));

            for payment in action.payments.iter() {
                serialized.append(&sc_format!("{}{}{}", payment.token_identifier, payment.token_nonce, payment.amount));
            }

            for arg in action.arguments.into_iter() {
                serialized.append(&sc_format!("{:x}", arg));
            }
        }

        self.crypto().keccak256(&serialized).as_managed_buffer().clone()
    }

    fn get_actual_permissions(&self, proposal: &Proposal<Self::Api>, actions: &ManagedVec<Action<Self::Api>>) -> ManagedVec<ManagedBuffer> {
        let proposer_id = self.users().get_user_id(&proposal.proposer);
        let mut actual_permissions = ManagedVec::new();

        for role in self.user_roles(proposer_id).iter() {
            for permission in self.policies(&role).keys() {
                let permission_details = self.permission_details(&permission).get();

                for action in actions.into_iter() {
                    if self.does_permission_apply_to_action(&permission_details, &action) && !actual_permissions.contains(&permission) {
                        actual_permissions.push(permission.clone());
                    }
                }
            }
        }

        actual_permissions
    }

    fn does_permission_apply_to_action(&self, permission_details: &PermissionDetails<Self::Api>, action: &Action<Self::Api>) -> bool {
        let mut applies = true;

        if applies && !permission_details.destination.is_zero() {
            applies = permission_details.destination == action.destination;
        }

        if applies && !permission_details.endpoint.is_empty() {
            applies = permission_details.endpoint == action.endpoint;
        }

        if applies && !permission_details.arguments.is_empty() {
            for (i, perm_arg) in permission_details.arguments.into_iter().enumerate() {
                if let Option::Some(arg_at_index) = action.arguments.try_get(i).as_deref() {
                    applies = arg_at_index == &perm_arg;
                } else {
                    applies = false;
                    break;
                }
            }
        }

        applies
    }

    fn has_sufficient_votes(&self, proposal: &Proposal<Self::Api>, quorum: &BigUint) -> bool {
        let total_votes = &proposal.votes_for + &proposal.votes_against;
        let vote_for_percent = &proposal.votes_for * &BigUint::from(100u64) / &total_votes;
        let vote_for_percent_to_pass = BigUint::from(50u64);

        vote_for_percent >= vote_for_percent_to_pass && &proposal.votes_for >= quorum
    }

    fn require_proposed_via_trusted_host(
        &self,
        trusted_host_id: &ManagedBuffer,
        content_hash: &ManagedBuffer,
        content_sig: ManagedBuffer,
        actions_hash: &ManagedBuffer,
        permissions: &ManagedVec<ManagedBuffer>
    ) {
        let proposer = self.blockchain().get_caller();
        let entity_address = self.blockchain().get_sc_address();
        let trusted_host_signature = ManagedByteArray::try_from(content_sig).unwrap();
        let mut trusted_host_signable = sc_format!("{:x}{:x}{:x}{:x}{:x}", proposer, entity_address, trusted_host_id, content_hash, actions_hash);

        for perm in permissions.into_iter() {
            trusted_host_signable.append(&sc_format!("{:x}", perm));
        }

        self.require_signed_by_trusted_host(&trusted_host_signable, &trusted_host_signature);
    }

    fn proposal_exists(&self, proposal_id: u64) -> bool {
        !self.proposals(proposal_id).is_empty()
    }
}
