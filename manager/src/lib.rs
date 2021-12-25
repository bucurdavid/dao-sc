#![no_std]

elrond_wasm::imports!();

mod factory;

#[elrond_wasm::contract]
pub trait Manager: factory::FactoryModule {
    #[init]
    fn init(&self, entity_template_address: ManagedAddress) {
        self.entity_templ_address().set(&entity_template_address);
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
        let token_properties = self.create_token_properties(dec_amount);

        Ok(self
            .send()
            .esdt_system_sc_proxy()
            .issue_fungible(issue_cost, &token_name, &token_ticker, &BigUint::from(0u32), token_properties)
            .async_call()
            .with_callback(self.callbacks().token_issue_callback(&initial_caller)))
    }

    #[endpoint(createEntityWithToken)]
    fn create_entity_with_token_endpoint(&self, token: TokenIdentifier) -> SCResult<()> {
        // 1. ui sets all the right properties
        // 2. ui transfers ownership
        require!(token.is_esdt(), "not an esdt");

        // check if user is owner and has all properties

        self.store_new_entity(token);

        Ok(())
    }

    #[endpoint(setup)]
    fn setup_endpoint(&self, token_id: TokenIdentifier) -> SCResult<AsyncCall> {
        let entity_address = self.get_entity_address(&token_id)?;
        let roles = [EsdtLocalRole::Mint, EsdtLocalRole::Burn];

        Ok(self
            .send()
            .esdt_system_sc_proxy()
            .set_special_roles(&entity_address, &token_id, (&roles[..]).into_iter().cloned())
            .async_call())
    }

    #[endpoint(finalize)]
    fn finalize_endpoint(&self, token_id: TokenIdentifier) -> SCResult<AsyncCall> {
        let entity_address = self.get_entity_address(&token_id)?;

        Ok(self
            .send()
            .esdt_system_sc_proxy()
            .transfer_ownership(&token_id, &entity_address.to_address())
            .async_call())
    }

    #[callback]
    fn token_issue_callback(&self, initial_caller: &ManagedAddress, #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => self.store_new_entity(token_id),
            ManagedAsyncCallResult::Err(_) => {
                let egld_returned = self.call_value().egld_value();
                if egld_returned > 0u32 {
                    self.send().direct_egld(&initial_caller, &egld_returned, &[]);
                }
            }
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

    fn store_new_entity(&self, token_id: TokenIdentifier) {
        let address = self.create_entity().unwrap_or_signal_error();
        self.entities_map().insert(token_id, address.clone());
    }

    fn get_entity_address(&self, token_id: &TokenIdentifier) -> SCResult<ManagedAddress> {
        require!(self.entities_map().contains_key(&token_id), "entity does not exist");
        Ok(self.entities_map().get(&token_id).unwrap())
    }

    #[storage_mapper("entities")]
    fn entities_map(&self) -> MapMapper<TokenIdentifier, ManagedAddress>;
}
