#![no_std]
#![feature(generic_associated_types)]

elrond_wasm::imports!();

pub mod config;
pub mod credits;
pub mod factory;
pub mod features;

#[elrond_wasm::contract]
pub trait Manager: config::ConfigModule + features::FeaturesModule + factory::FactoryModule + credits::CreditsModule {
    #[init]
    fn init(
        &self,
        entity_template_address: ManagedAddress,
        trusted_host_address: ManagedAddress,
        cost_token: TokenIdentifier,
        cost_entity_creation: BigUint,
    ) {
        self.entity_templ_address().set_if_empty(&entity_template_address);
        self.trusted_host_address().set(&trusted_host_address);
        self.cost_token_id().set_if_empty(&cost_token);
        self.cost_creation_amount().set(&cost_entity_creation);
    }

    #[payable("*")]
    #[endpoint(deposit)]
    fn deposit_endpoint(&self) {}

    #[payable("*")]
    #[endpoint(executeTicket)]
    fn execute_ticket_endpoint(&self, ticket_type: ManagedBuffer, ticket_id: ManagedBuffer) {
        require!(!ticket_type.is_empty(), "ticket type is required");
        require!(!ticket_id.is_empty(), "ticket id is required");
    }

    #[payable("*")]
    #[endpoint(createEntity)]
    fn create_entity_endpoint(&self, features: MultiValueManagedVec<ManagedBuffer>) -> ManagedAddress {
        let payment = self.call_value().single_esdt();

        require!(payment.token_identifier == self.cost_token_id().get(), "invalid cost token");
        require!(payment.amount >= self.cost_creation_amount().get(), "invalid cost amount");

        let entity_address = self.create_entity();

        self.set_features(&entity_address, features.into_vec());
        self.recalculate_daily_cost(&entity_address);
        self.entities().insert(entity_address.clone());

        entity_address
    }

    #[endpoint(upgradeEntity)]
    fn upgrade_entity_endpoint(&self, entity_address: ManagedAddress) {
        self.recalculate_daily_cost(&entity_address);
        self.upgrade_entity(entity_address);
    }

    #[endpoint(setFeatures)]
    fn set_features_endpoint(&self, features: MultiValueManagedVec<ManagedBuffer>) {
        let caller_entity_address = self.blockchain().get_caller();
        let features = features.into_vec();

        self.require_entity_exists(&caller_entity_address);
        self.set_features(&caller_entity_address, features);
        self.recalculate_daily_cost(&caller_entity_address);
    }
}
