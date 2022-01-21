#![no_std]

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
    #[endpoint(createEntityToken)]
    fn create_entity_token_endpoint(
        &self,
        token_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
        #[payment] issue_cost: BigUint,
    ) -> SCResult<AsyncCall> {
        require!(num_decimals <= 18 as usize, "invalid token decimals");
        let initial_caller = self.blockchain().get_caller();

        Ok(self
            .issue_token(token_name, token_ticker, num_decimals, issue_cost)
            .with_callback(self.callbacks().token_issue_callback(&initial_caller)))
    }

    #[only_owner]
    #[endpoint(createEntityWithToken)]
    fn create_entity_with_token_endpoint(&self, token_id: TokenIdentifier) -> SCResult<()> {
        require!(token_id.is_valid_esdt_identifier(), "not an esdt");

        let caller = self.blockchain().get_caller();

        self.setup_owner_token(&caller).set(&token_id);
        Ok(())
    }

    #[payable("*")]
    #[endpoint(createEntity)]
    fn create_entity_endpoint(
        &self,
        #[payment_token] cost_token_id: TokenIdentifier,
        #[payment_amount] cost_amount: BigUint,
        token_id: TokenIdentifier,
        #[var_args] feature_names: VarArgs<ManagedBuffer>,
    ) -> SCResult<AsyncCall> {
        self.require_caller_is_temp_owner(&token_id)?;

        let entity_address = self.create_entity(&token_id)?;

        self.entities_map().insert(token_id.clone(), entity_address.clone());

        self.enable_entity_features(&entity_address, feature_names);

        self.burn_entity_creation_cost_tokens(cost_token_id, cost_amount)?;

        self.send_control_token(&token_id);

        Ok(self.set_entity_edst_roles(&token_id, &entity_address))
    }

    #[endpoint(finalizeEntity)]
    fn finalize_entity_endpoint(&self, token_id: TokenIdentifier) -> SCResult<AsyncCall> {
        self.require_caller_is_temp_owner(&token_id)?;

        let caller = self.blockchain().get_caller();
        let entity_address = self.get_entity_address(&token_id)?;

        self.setup_owner_token(&caller).clear();

        Ok(self.transfer_entity_esdt_ownership(&token_id, &entity_address))
    }

    #[payable("*")]
    #[callback]
    fn token_issue_callback(
        &self,
        initial_caller: &ManagedAddress,
        #[payment_token] payment_token: TokenIdentifier,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(_) => self.setup_owner_token(&initial_caller).set(&payment_token),
            ManagedAsyncCallResult::Err(_) => self.send_back_egld(&initial_caller),
        }
    }

    #[endpoint(upgradeEntity)]
    fn upgrade_entity_endpoint(&self, token_id: TokenIdentifier) -> SCResult<()> {
        let entity_address = self.get_entity_address(&token_id)?;

        self.upgrade_entity(&entity_address)?;

        Ok(())
    }

    #[view(getEntityAddress)]
    fn get_entity_address_view(&self, token_id: TokenIdentifier) -> ManagedAddress {
        self.entities_map().get(&token_id).unwrap_or_default()
    }

    fn get_entity_address(&self, token_id: &TokenIdentifier) -> SCResult<ManagedAddress> {
        require!(self.entities_map().contains_key(&token_id), "entity does not exist");
        Ok(self.entities_map().get(&token_id).unwrap())
    }

    fn send_back_egld(&self, initial_caller: &ManagedAddress) {
        let egld_returned = self.call_value().egld_value();
        if egld_returned > 0u32 {
            self.send().direct_egld(&initial_caller, &egld_returned, &[]);
        }
    }

    fn require_caller_is_temp_owner(&self, token_id: &TokenIdentifier) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        let temp_owner_token_id = self.setup_owner_token(&caller).get();
        require!(&temp_owner_token_id == token_id, "token not in setup");
        Ok(())
    }

    #[storage_mapper("entities")]
    fn entities_map(&self) -> MapMapper<TokenIdentifier, ManagedAddress>;

    #[view(getSetupOwnerToken)]
    #[storage_mapper("setup_owner_token")]
    fn setup_owner_token(&self, owner: &ManagedAddress) -> SingleValueMapper<TokenIdentifier>;
}
