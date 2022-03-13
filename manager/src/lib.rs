#![no_std]
#![feature(generic_associated_types)]

elrond_wasm::imports!();

mod cost;
mod esdt;
mod factory;

#[elrond_wasm::contract]
pub trait Manager: factory::FactoryModule + esdt::EsdtModule + cost::CostModule {
    #[init]
    fn init(&self, entity_template_address: ManagedAddress, cost_token: TokenIdentifier, cost_entity_creation: BigUint) {
        self.init_factory_module(entity_template_address);
        self.init_cost_module(cost_token, cost_entity_creation);
    }

    #[payable("EGLD")]
    #[endpoint(deposit)]
    fn deposit_endpoint(&self) {}

    #[payable("EGLD")]
    #[endpoint(createEntityToken)]
    fn create_entity_token_endpoint(&self, token_name: ManagedBuffer, token_ticker: ManagedBuffer, amount: BigUint, #[payment] issue_cost: BigUint) {
        require!(amount > 0, "amount must be greater than zero");
        let initial_caller = self.blockchain().get_caller();

        self.issue_token(&token_name, &token_ticker, &amount, &issue_cost)
            .with_callback(self.callbacks().token_issue_callback(&initial_caller))
            .call_and_exit()
    }

    #[payable("*")]
    #[endpoint(createEntityWithToken)]
    fn create_entity_with_token_endpoint(&self, #[payment_token] payment_token: TokenIdentifier, #[payment_amount] payment_amount: BigUint) {
        let caller = self.blockchain().get_caller();

        self.setup_token_id(&caller).set(&payment_token);
        self.setup_token_amount(&caller).set(&payment_amount);
    }

    #[payable("*")]
    #[endpoint(createEntity)]
    fn create_entity_endpoint(
        &self,
        #[payment_token] cost_token_id: TokenIdentifier,
        #[payment_amount] cost_amount: BigUint,
        token_id: TokenIdentifier,
        #[var_args] feature_names: MultiValueEncoded<ManagedBuffer>,
    ) {
        self.require_caller_is_setup_owner(&token_id);

        let caller = self.blockchain().get_caller();
        let initial_tokens = self.setup_token_amount(&caller).get();

        require!(initial_tokens > 0, "setup token is not available");

        let entity_address = self.create_entity(&token_id, &initial_tokens);

        self.entities_map().insert(token_id.clone(), entity_address.clone());

        self.enable_entity_features(&entity_address, feature_names);

        self.burn_entity_creation_cost_tokens(cost_token_id, cost_amount);

        self.set_entity_edst_roles(&token_id, &entity_address).call_and_exit()
    }

    #[endpoint(finalizeEntity)]
    fn finalize_entity_endpoint(&self, token_id: TokenIdentifier) {
        self.require_caller_is_setup_owner(&token_id);

        let caller = self.blockchain().get_caller();
        let entity_address = self.get_entity_address(&token_id);

        self.setup_token_id(&caller).clear();
        self.setup_token_amount(&caller).clear();

        self.transfer_entity_esdt_ownership(&token_id, &entity_address).call_and_exit()
    }

    #[payable("*")]
    #[callback]
    fn token_issue_callback(
        &self,
        initial_caller: &ManagedAddress,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {
                self.setup_token_id(&initial_caller).set(&payment_token);
                self.setup_token_amount(&initial_caller).set(&payment_amount);
                self.send().direct(&initial_caller, &payment_token, 0, &payment_amount, &[]);
            }
            ManagedAsyncCallResult::Err(_) => self.send_back_egld(&initial_caller),
        }
    }

    #[endpoint(upgradeEntity)]
    fn upgrade_entity_endpoint(&self, token_id: TokenIdentifier) {
        self.upgrade_entity(self.get_entity_address(&token_id));
    }

    #[view(getEntityAddress)]
    fn get_entity_address_view(&self, token_id: TokenIdentifier) -> ManagedAddress {
        self.entities_map().get(&token_id).unwrap_or_default()
    }

    fn get_entity_address(&self, token_id: &TokenIdentifier) -> ManagedAddress {
        require!(self.entities_map().contains_key(&token_id), "entity does not exist");

        self.entities_map().get(&token_id).unwrap()
    }

    fn send_back_egld(&self, initial_caller: &ManagedAddress) {
        let egld_returned = self.call_value().egld_value();
        if egld_returned > 0u32 {
            self.send().direct_egld(&initial_caller, &egld_returned, &[]);
        }
    }

    fn require_caller_is_setup_owner(&self, token_id: &TokenIdentifier) {
        let caller = self.blockchain().get_caller();
        let temp_owner_token_id = self.setup_token_id(&caller).get();
        require!(&temp_owner_token_id == token_id, "token not in setup");
    }

    #[storage_mapper("entities")]
    fn entities_map(&self) -> MapMapper<TokenIdentifier, ManagedAddress>;

    #[view(getSetupToken)]
    #[storage_mapper("setup:token_id")]
    fn setup_token_id(&self, owner: &ManagedAddress) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("setup:token_amount")]
    fn setup_token_amount(&self, owner: &ManagedAddress) -> SingleValueMapper<BigUint>;
}
