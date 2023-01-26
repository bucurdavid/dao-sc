multiversx_sc::imports!();

use crate::config;

const DEFAULT_DECIMALS: usize = 18usize;

#[multiversx_sc::module]
pub trait TokenModule: config::ConfigModule {
    fn issue_gov_token(&self, token_name: ManagedBuffer, token_ticker: ManagedBuffer, supply: BigUint) -> AsyncCall {
        require!(supply > 0, "amount must be greater than zero");

        let properties = FungibleTokenProperties {
            num_decimals: DEFAULT_DECIMALS,
            can_burn: false,
            can_mint: false,
            can_freeze: true,
            can_wipe: true,
            can_pause: true,
            can_change_owner: false,
            can_upgrade: true,
            can_add_special_roles: true,
        };

        self.send()
            .esdt_system_sc_proxy()
            .issue_fungible(self.call_value().egld_value(), &token_name, &token_ticker, &supply, properties)
            .async_call()
    }

    fn send_received_egld(&self, to: &ManagedAddress) {
        let egld_received = self.call_value().egld_value();
        if egld_received > 0 {
            self.send().direct_egld(&to, &egld_received);
        }
    }
}
