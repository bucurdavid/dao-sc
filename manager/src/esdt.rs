elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait EsdtModule {
    fn issue_token(&self, name: ManagedBuffer, ticker: ManagedBuffer, num_decimals: usize, cost: BigUint) -> AsyncCall {
        let properties = FungibleTokenProperties {
            num_decimals,
            can_burn: false,
            can_mint: false,
            can_freeze: true,
            can_wipe: true,
            can_pause: true,
            can_change_owner: false,
            can_upgrade: false,
            can_add_special_roles: true,
        };

        self.send()
            .esdt_system_sc_proxy()
            .issue_fungible(cost, &name, &ticker, &BigUint::from(0u32), properties)
            .async_call()
    }

    fn set_entity_edst_roles(&self, token_id: &TokenIdentifier, entity_address: &ManagedAddress) -> AsyncCall {
        let roles = [EsdtLocalRole::Mint, EsdtLocalRole::Burn];

        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(&entity_address, &token_id, (&roles[..]).into_iter().cloned())
            .async_call()
    }

    fn transfer_entity_edst_ownership(&self, token_id: &TokenIdentifier, entity_address: &ManagedAddress) -> AsyncCall {
        self.send()
            .esdt_system_sc_proxy()
            .transfer_ownership(&token_id, &entity_address.to_address())
            .async_call()
    }
}
