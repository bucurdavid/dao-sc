#![no_std]

elrond_wasm::imports!();

mod factory;

#[elrond_wasm::contract]
pub trait Manager: factory::FactoryModule {
    #[init]
    fn init(&self, entity_template_address: ManagedAddress) {
        self.init_factory(entity_template_address);
    }

    #[payable("EGLD")]
    #[endpoint(createEntity)]
    fn create_entity_endpoint(
        &self,
        token_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        dec_amount: usize,
        #[payment] issue_cost: BigUint,
    ) -> SCResult<AsyncCall> {
        let initial_caller = self.blockchain().get_caller();

        Ok(self
            .issue_token(token_name, token_ticker, dec_amount, issue_cost)
            .with_callback(self.callbacks().token_issue_callback(&initial_caller)))
    }

    #[only_owner]
    #[endpoint(createEntityWithToken)]
    fn create_entity_with_token_endpoint(&self, token: TokenIdentifier) -> SCResult<()> {
        require!(token.is_valid_esdt_identifier(), "not an esdt");

        let caller = self.blockchain().get_caller();

        self.store_new_entity(&caller, token)?;
        Ok(())
    }

    #[endpoint(setupEntity)]
    fn setup_entity_endpoint(&self, token_id: TokenIdentifier) -> SCResult<AsyncCall> {
        self.require_caller_is_temp_owner(&token_id)?;
        let entity_address = self.get_entity_address(&token_id)?;

        // burn super esdts

        Ok(self.set_entity_edst_roles(&token_id, &entity_address))
    }

    #[endpoint(finalize)]
    fn finalize_endpoint(&self, token_id: TokenIdentifier) -> SCResult<AsyncCall> {
        self.require_caller_is_temp_owner(&token_id)?;
        let entity_address = self.get_entity_address(&token_id)?;
        let caller = self.blockchain().get_caller();

        self.temp_setup_owner().remove(&caller);

        Ok(self.transfer_entity_edst_ownership(&token_id, &entity_address))
    }

    #[callback]
    fn token_issue_callback(&self, initial_caller: &ManagedAddress, #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>) -> SCResult<()> {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => self.store_new_entity(initial_caller, token_id),
            ManagedAsyncCallResult::Err(_) => Ok(self.send_back_egld(&initial_caller)),
        }
    }

    #[endpoint(upgradeEntity)]
    fn upgrade_entity_entity(&self) -> SCResult<()> {
        Ok(())
    }

    #[view(getEntityAddress)]
    fn get_entity_address_view(&self, token_id: TokenIdentifier) -> ManagedAddress {
        self.entities_map().get(&token_id).unwrap_or_else(|| ManagedAddress::zero())
    }

    fn store_new_entity(&self, caller: &ManagedAddress, token_id: TokenIdentifier) -> SCResult<()> {
        let address = self.create_entity()?;
        self.entities_map().insert(token_id.clone(), address.clone());
        self.temp_setup_owner().insert(caller.clone(), token_id);
        Ok(())
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
        let temp_owner_token_id = self.temp_setup_owner().get(&caller).unwrap_or(TokenIdentifier::egld());
        require!(temp_owner_token_id == *token_id, "not in setup");
        Ok(())
    }

    #[storage_mapper("entities")]
    fn entities_map(&self) -> MapMapper<TokenIdentifier, ManagedAddress>;

    #[storage_mapper("temp_setup_owner")]
    fn temp_setup_owner(&self) -> MapMapper<ManagedAddress, TokenIdentifier>;
}
