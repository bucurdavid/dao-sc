#![no_std]
#![feature(generic_associated_types)]

elrond_wasm::imports!();

pub mod config;
pub mod credits;
pub mod esdt;
pub mod factory;
pub mod features;

#[elrond_wasm::contract]
pub trait Manager: config::ConfigModule + features::FeaturesModule + factory::FactoryModule + esdt::EsdtModule + credits::CreditsModule {
    #[init]
    fn init(&self, entity_template_address: ManagedAddress, trusted_host_address: ManagedAddress, cost_token: TokenIdentifier, cost_entity_creation: BigUint) {
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
    fn execute_ticket_endpoint(&self, ticket_id: ManagedBuffer) {
        require!(ticket_id.len() > 0, "ticket id is required");
    }

    #[payable("EGLD")]
    #[endpoint(createEntityToken)]
    fn create_entity_token_endpoint(&self, token_name: ManagedBuffer, token_ticker: ManagedBuffer, amount: BigUint) {
        let issue_cost = self.call_value().egld_value();
        let initial_caller = self.blockchain().get_caller();

        require!(amount > 0, "amount must be greater than zero");

        self.issue_token(&token_name, &token_ticker, &amount, &issue_cost)
            .with_callback(self.callbacks().token_issue_callback(&initial_caller))
            .call_and_exit()
    }

    #[payable("*")]
    #[endpoint(registerEntityToken)]
    fn register_entity_token_endpoint(&self, supply: BigUint) {
        let caller = self.blockchain().get_caller();
        let proof = self.call_value().single_esdt();

        self.setup_token_id(&caller).set(&proof.token_identifier);
        self.setup_token_supply(&caller).set(&supply);

        self.send().direct_esdt(&caller, &proof.token_identifier, proof.token_nonce, &proof.amount);
    }

    #[payable("*")]
    #[endpoint(createEntity)]
    fn create_entity_endpoint(&self, features: MultiValueManagedVec<ManagedBuffer>) {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();

        require!(payment.token_identifier == self.cost_token_id().get(), "invalid cost token");
        require!(payment.amount >= self.cost_creation_amount().get(), "invalid cost amount");
        self.require_caller_in_setup(&caller);

        let token_id = self.setup_token_id(&caller).get();
        let initial_supply = self.setup_token_supply(&caller).get();
        let features = features.into_vec();

        require!(initial_supply > 0, "setup token is not available");

        let entity_address = self.create_entity(&token_id, &initial_supply, &features);

        self.set_features(&entity_address, features);
        self.entities().insert(entity_address.clone());
        self.setup_token_entity_history(&token_id).set(entity_address.clone());
        self.recalculate_daily_cost(&entity_address);
        self.set_entity_edst_roles(&token_id, &entity_address).call_and_exit();
    }

    #[endpoint(finalizeEntity)]
    fn finalize_entity_endpoint(&self) {
        let caller = self.blockchain().get_caller();
        self.require_caller_in_setup(&caller);

        let token_id = self.setup_token_id(&caller).get();
        let entity_address = self.setup_token_entity_history(&token_id).get();

        self.setup_token_id(&caller).clear();
        self.setup_token_supply(&caller).clear();
        self.transfer_entity_esdt_ownership(&token_id, &entity_address).call_and_exit();
    }

    #[payable("*")]
    #[callback]
    fn token_issue_callback(&self, initial_caller: &ManagedAddress, #[call_result] result: ManagedAsyncCallResult<()>) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {
                let payment = self.call_value().single_esdt();
                self.setup_token_id(&initial_caller).set(&payment.token_identifier);
                self.setup_token_supply(&initial_caller).set(&payment.amount);
                self.send().direct_esdt(&initial_caller, &payment.token_identifier, 0, &payment.amount);
            }
            ManagedAsyncCallResult::Err(_) => self.send_back_egld(&initial_caller),
        }
    }

    #[endpoint(upgradeEntity)]
    fn upgrade_entity_endpoint(&self, entity_address: ManagedAddress) {
        self.recalculate_daily_cost(&entity_address);
        self.upgrade_entity(entity_address);
    }

    #[endpoint(setFeatures)]
    fn set_features_endpoint(&self, features: MultiValueManagedVec<ManagedBuffer>) {
        let caller_entity_address = self.blockchain().get_caller();
        self.require_entity_exists(&caller_entity_address);
        self.set_features(&caller_entity_address, features.into_vec());
        self.recalculate_daily_cost(&caller_entity_address);
    }

    #[only_owner]
    #[endpoint(clearSetup)]
    fn clear_setup_endpoint(&self, address: ManagedAddress) {
        self.setup_token_id(&address).clear();
        self.setup_token_supply(&address).clear();
    }

    fn require_caller_in_setup(&self, caller: &ManagedAddress) {
        require!(!self.setup_token_id(&caller).is_empty(), "not in setup: token");
        require!(!self.setup_token_supply(&caller).is_empty(), "not in setup: supply");
    }

    fn send_back_egld(&self, initial_caller: &ManagedAddress) {
        let egld_returned = self.call_value().egld_value();
        if egld_returned > 0 {
            self.send().direct_egld(&initial_caller, &egld_returned);
        }
    }

    #[view(getSetupToken)]
    #[storage_mapper("setup:token_id")]
    fn setup_token_id(&self, owner: &ManagedAddress) -> SingleValueMapper<TokenIdentifier>;

    #[view(getSetupTokenAmount)]
    #[storage_mapper("setup:token_supply")]
    fn setup_token_supply(&self, owner: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[view(getSetupTokenHistoryEntityAddress)]
    #[storage_mapper("setup:token_entity_history")]
    fn setup_token_entity_history(&self, token_id: &TokenIdentifier) -> SingleValueMapper<ManagedAddress>;
}
