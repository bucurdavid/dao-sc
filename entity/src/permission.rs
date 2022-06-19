elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::config;

const ROLE_BUILTIN_LEADER: &[u8] = b"leader";

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct PermissionDetails<M: ManagedTypeApi> {
    pub destination: ManagedAddress<M>,
    pub endpoint: ManagedBuffer<M>,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct Policy<M: ManagedTypeApi> {
    pub permission_name: ManagedBuffer<M>,
    pub method: PolicyMethod,
    pub quorum: BigUint<M>,
    pub voting_period_minutes: u32,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, Copy, PartialEq, Debug)]
pub enum PolicyMethod {
    Weight,
    One,
    All,
    Quorum,
}

#[elrond_wasm::module]
pub trait PermissionModule: config::ConfigModule {
    fn init_permission_module(&self, opt_leader: OptionalValue<ManagedAddress>) {
        let has_initialized = !self.roles().is_empty() || !self.permissions().is_empty();

        if has_initialized {
            return;
        }

        self.create_role(ManagedBuffer::from(ROLE_BUILTIN_LEADER));

        if let OptionalValue::Some(leader) = opt_leader {
            self.assign_role(leader, ManagedBuffer::from(ROLE_BUILTIN_LEADER));
        }
    }

    fn create_role(&self, role_name: ManagedBuffer) {
        let created = self.roles().insert(role_name);

        require!(created, "role already exists");
    }

    fn has_role(&self, address: ManagedAddress, role_name: ManagedBuffer) -> bool {
        let user_id = self.users().get_user_id(&address);

        if user_id == 0 {
            return false;
        }

        self.user_roles(user_id).contains(&role_name)
    }

    fn assign_role(&self, address: ManagedAddress, role_name: ManagedBuffer) {
        require!(self.roles().contains(&role_name), "role does not exist");

        let user_id = self.users().get_or_create_user(&address);
        let added = self.user_roles(user_id).insert(role_name);

        require!(added, "user already has role");
    }

    fn create_permission(&self, permission_name: ManagedBuffer, destination: ManagedAddress, endpoint: ManagedBuffer) {
        let created = self.permissions().insert(permission_name.clone());

        require!(created, "permission already exists");

        self.permission_details(&permission_name).set(PermissionDetails {
            destination,
            endpoint,
        });
    }

    #[view(getPermissions)]
    fn get_permissions_view(&self) -> MultiValueEncoded<MultiValue3<ManagedBuffer, ManagedAddress, ManagedBuffer>> {
        let mut permissions = MultiValueEncoded::new();

        for permission_name in self.permissions().iter() {
            let permission_details = self.permission_details(&permission_name).get();
            permissions.push((permission_name, permission_details.destination, permission_details.endpoint).into());
        }

        permissions
    }

    #[view(getRoles)]
    #[storage_mapper("roles")]
    fn roles(&self) -> UnorderedSetMapper<ManagedBuffer<Self::Api>>;

    #[view(getUserRoles)]
    #[storage_mapper("user_roles")]
    fn user_roles(&self, user_id: usize) -> UnorderedSetMapper<ManagedBuffer<Self::Api>>;

    #[storage_mapper("permissions")]
    fn permissions(&self) -> UnorderedSetMapper<ManagedBuffer<Self::Api>>;

    #[storage_mapper("permission_details")]
    fn permission_details(&self, permission_name: &ManagedBuffer) -> SingleValueMapper<PermissionDetails<Self::Api>>;

    #[view(getPolicies)]
    #[storage_mapper("policies")]
    fn policies(&self, role_name: &ManagedBuffer) -> MapMapper<ManagedBuffer<Self::Api>, Policy<Self::Api>>;
}
