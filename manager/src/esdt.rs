elrond_wasm::imports!();

const CONTROL_TOKEN_AMOUNT: u32 = 1u32;

#[elrond_wasm::module]
pub trait EsdtModule {
    fn issue_token(&self, name: ManagedBuffer, ticker: ManagedBuffer, num_decimals: usize, cost: BigUint) -> AsyncCall {
        let initial_supply = BigUint::from(CONTROL_TOKEN_AMOUNT);
        let properties = FungibleTokenProperties {
            can_burn: false,
            can_mint: false,
            num_decimals,
            can_freeze: true,
            can_wipe: true,
            can_pause: true,
            can_change_owner: true,
            can_upgrade: false,
            can_add_special_roles: true,
        };

        self.send()
            .esdt_system_sc_proxy()
            .issue_fungible(cost, &name, &ticker, &initial_supply, properties)
            .async_call()
    }

    fn set_entity_edst_roles(&self, token_id: &TokenIdentifier, entity_address: &ManagedAddress) -> AsyncCall {
        let roles = [EsdtLocalRole::Mint, EsdtLocalRole::Burn];

        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(&entity_address, &token_id, (&roles[..]).into_iter().cloned())
            .async_call()
    }

    fn transfer_entity_esdt_ownership(&self, token_id: &TokenIdentifier, entity_address: &ManagedAddress) -> AsyncCall {
        self.send()
            .esdt_system_sc_proxy()
            .transfer_ownership(&token_id, &entity_address.to_address())
            .async_call()
    }

    fn send_control_token(&self, token_id: &TokenIdentifier) {
        let caller = self.blockchain().get_caller();
        let amount = BigUint::from(CONTROL_TOKEN_AMOUNT);
        self.send().direct(&caller, &token_id, 0, &amount, &[]);
    }
}
