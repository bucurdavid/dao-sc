elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::config;

pub const ROLE_BUILTIN_LEADER: &[u8] = b"leader";

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct PermissionDetails<M: ManagedTypeApi> {
    pub value: BigUint<M>,
    pub destination: ManagedAddress<M>,
    pub endpoint: ManagedBuffer<M>,
    pub arguments: ManagedVec<M, ManagedBuffer<M>>,
    pub payments: ManagedVec<M, EsdtTokenPayment<M>>,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem)]
pub struct Policy<M: ManagedTypeApi> {
    pub method: PolicyMethod,
    pub quorum: BigUint<M>,
    pub voting_period_minutes: usize,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, Copy, PartialEq, Debug, ManagedVecItem)]
pub enum PolicyMethod {
    Weight,
    One,
    All,
    Quorum,
}

impl PolicyMethod {
    pub fn to_name(&self) -> &[u8] {
        match self {
            PolicyMethod::Weight => b"weight",
            PolicyMethod::One => b"one",
            PolicyMethod::All => b"all",
            PolicyMethod::Quorum => b"quorum",
        }
    }
}

#[elrond_wasm::module]
pub trait PermissionModule: config::ConfigModule {
    fn init_permission_module(&self, leader: ManagedAddress) {
        self.create_role(ManagedBuffer::from(ROLE_BUILTIN_LEADER));
        self.assign_role(leader, ManagedBuffer::from(ROLE_BUILTIN_LEADER));
    }

    #[endpoint(createRole)]
    fn create_role_endpoint(&self, role_name: ManagedBuffer) {
        self.require_caller_self();
        self.create_role(role_name);
    }

    #[endpoint(removeRole)]
    fn remove_role_endpoint(&self, role_name: ManagedBuffer) {
        self.require_caller_self();
        self.remove_role(role_name);
    }

    #[endpoint(assignRole)]
    fn assign_role_endpoint(&self, role_name: ManagedBuffer, address: ManagedAddress) {
        self.require_caller_self();
        self.assign_role(address, role_name);
    }

    #[endpoint(unassignRole)]
    fn unassign_role_endpoint(&self, role_name: ManagedBuffer, address: ManagedAddress) {
        self.require_caller_self();
        self.unassign_role(address, role_name);
    }

    #[endpoint(createPermission)]
    fn create_permission_endpoint(
        &self,
        permission_name: ManagedBuffer,
        value: BigUint,
        destination: ManagedAddress,
        endpoint: ManagedBuffer,
        payments_multi: MultiValueManagedVec<EsdtTokenPaymentMultiValue>,
    ) {
        self.require_caller_self();

        let mut payments = ManagedVec::new();

        for payment in payments_multi.iter() {
            payments.push(payment.into_esdt_token_payment());
        }

        self.create_permission(permission_name, value, destination, endpoint, ManagedVec::new(), payments);
    }

    #[endpoint(createPolicyWeighted)]
    fn create_policy_weighted_endpoint(&self, role_name: ManagedBuffer, permission_name: ManagedBuffer, quorum: BigUint, voting_period_minutes: usize) {
        self.require_caller_self();
        self.create_policy(role_name, permission_name, PolicyMethod::Weight, quorum, voting_period_minutes);
    }

    #[endpoint(createPolicyForOne)]
    fn create_policy_one_endpoint(&self, role_name: ManagedBuffer, permission_name: ManagedBuffer) {
        self.require_caller_self();
        self.create_policy(role_name, permission_name, PolicyMethod::One, BigUint::from(1u64), 0);
    }

    #[endpoint(createPolicyForAll)]
    fn create_policy_all_endpoint(&self, role_name: ManagedBuffer, permission_name: ManagedBuffer) {
        self.require_caller_self();
        self.create_policy(role_name, permission_name, PolicyMethod::All, BigUint::zero(), self.voting_period_in_minutes().get());
    }

    #[endpoint(createPolicyQuorum)]
    fn create_policy_quorum_endpoint(&self, role_name: ManagedBuffer, permission_name: ManagedBuffer, quorum: usize) {
        self.require_caller_self();
        self.create_policy(
            role_name,
            permission_name,
            PolicyMethod::Quorum,
            BigUint::from(quorum),
            self.voting_period_in_minutes().get(),
        );
    }

    #[view(getUserRoles)]
    fn get_user_roles_view(&self, address: ManagedAddress) -> MultiValueEncoded<ManagedBuffer> {
        let user_id = self.users().get_user_id(&address);
        let mut roles = MultiValueEncoded::new();

        if user_id == 0 {
            return roles;
        }

        for role in self.user_roles(user_id).iter() {
            roles.push(role);
        }

        roles
    }

    #[view(getPermissions)]
    fn get_permissions_view(
        &self,
    ) -> MultiValueEncoded<
        MultiValue8<
            ManagedBuffer,
            BigUint,
            ManagedAddress,
            ManagedBuffer,
            usize,
            MultiValueManagedVec<ManagedBuffer>,
            usize,
            MultiValueManagedVec<EsdtTokenPaymentMultiValue>,
        >,
    > {
        let mut permissions = MultiValueEncoded::new();

        for permission_name in self.permissions().iter() {
            let perm = self.permission_details(&permission_name).get();
            let mut payments_multi = MultiValueManagedVec::new();

            for payment in perm.payments.into_iter() {
                payments_multi.push(payment.into_multi_value());
            }

            permissions.push(
                (
                    permission_name,
                    perm.value,
                    perm.destination,
                    perm.endpoint,
                    perm.arguments.len(),
                    MultiValueManagedVec::from(perm.arguments),
                    perm.payments.len(),
                    payments_multi,
                )
                    .into(),
            );
        }

        permissions
    }

    #[view(getPolicies)]
    fn get_policies_view(&self, role_name: ManagedBuffer) -> MultiValueEncoded<MultiValue4<ManagedBuffer, ManagedBuffer, BigUint, usize>> {
        let mut policies = MultiValueEncoded::new();

        for (permission_name, policy) in self.policies(&role_name).iter() {
            policies.push(
                (
                    permission_name,
                    ManagedBuffer::from(policy.method.to_name()),
                    policy.quorum,
                    policy.voting_period_minutes,
                )
                    .into(),
            );
        }

        policies
    }

    fn create_role(&self, role_name: ManagedBuffer) {
        let created = self.roles().insert(role_name);

        require!(created, "role already exists")
    }

    fn remove_role(&self, role_name: ManagedBuffer) {
        require!(self.roles().contains(&role_name), "role does not exist");

        self.roles().swap_remove(&role_name);
        self.roles_member_amount(&role_name).set(0);

        for user_id in 1..=self.users().get_user_count() {
            self.user_roles(user_id).swap_remove(&role_name);
        }
    }

    fn assign_role(&self, address: ManagedAddress, role_name: ManagedBuffer) {
        require!(self.roles().contains(&role_name), "role does not exist");

        let user_id = self.users().get_or_create_user(&address);

        if self.user_roles(user_id).insert(role_name.clone()) {
            self.roles_member_amount(&role_name).update(|current| *current += 1);
        }
    }

    fn unassign_role(&self, address: ManagedAddress, role_name: ManagedBuffer) {
        require!(self.roles().contains(&role_name), "role does not exist");

        let user_id = self.users().get_or_create_user(&address);

        if self.user_roles(user_id).swap_remove(&role_name) {
            self.roles_member_amount(&role_name).update(|current| *current -= 1);
        }
    }

    fn create_permission(
        &self,
        permission_name: ManagedBuffer,
        value: BigUint,
        destination: ManagedAddress,
        endpoint: ManagedBuffer,
        arguments: ManagedVec<ManagedBuffer>,
        payments: ManagedVec<EsdtTokenPayment>,
    ) {
        self.permissions().insert(permission_name.clone());

        self.permission_details(&permission_name).set(PermissionDetails {
            value,
            destination,
            endpoint,
            arguments,
            payments,
        });
    }

    fn create_policy(&self, role_name: ManagedBuffer, permission_name: ManagedBuffer, method: PolicyMethod, quorum: BigUint, voting_period_minutes: usize) {
        require!(self.roles().contains(&role_name), "role does not exist");
        require!(self.permissions().contains(&permission_name), "permission does not exist");
        require!(!self.policies(&role_name).contains_key(&permission_name), "policy already exists");

        self.policies(&role_name).insert(
            permission_name,
            Policy {
                method,
                quorum,
                voting_period_minutes,
            },
        );
    }

    fn has_role(&self, address: &ManagedAddress, role_name: &ManagedBuffer) -> bool {
        let user_id = self.users().get_user_id(&address);

        if user_id == 0 {
            return false;
        }

        self.user_roles(user_id).contains(&role_name)
    }

    fn has_token_weighted_policy(&self, policies: &ManagedVec<Policy<Self::Api>>) -> bool {
        policies.iter().find(|p| p.method == PolicyMethod::Weight).is_some()
    }

    fn is_leaderless(&self) -> bool {
        !self.roles().contains(&ManagedBuffer::from(ROLE_BUILTIN_LEADER))
    }

    fn has_leader_role(&self, address: &ManagedAddress) -> bool {
        let leader_role = ManagedBuffer::from(ROLE_BUILTIN_LEADER);

        self.has_role(&address, &leader_role)
    }

    fn require_caller_has_leader_role(&self) {
        let caller = self.blockchain().get_caller();
        require!(self.has_leader_role(&caller), "caller must be leader");
    }

    #[view(getRoles)]
    #[storage_mapper("roles")]
    fn roles(&self) -> UnorderedSetMapper<ManagedBuffer<Self::Api>>;

    #[view(getRoleMemberAmount)]
    #[storage_mapper("roles_member_amount")]
    fn roles_member_amount(&self, role_name: &ManagedBuffer) -> SingleValueMapper<usize>;

    #[storage_mapper("user_roles")]
    fn user_roles(&self, user_id: usize) -> UnorderedSetMapper<ManagedBuffer<Self::Api>>;

    #[storage_mapper("permissions")]
    fn permissions(&self) -> UnorderedSetMapper<ManagedBuffer<Self::Api>>;

    #[storage_mapper("permission_details")]
    fn permission_details(&self, permission_name: &ManagedBuffer) -> SingleValueMapper<PermissionDetails<Self::Api>>;

    #[storage_mapper("policies")]
    fn policies(&self, role_name: &ManagedBuffer) -> MapMapper<ManagedBuffer<Self::Api>, Policy<Self::Api>>;
}
