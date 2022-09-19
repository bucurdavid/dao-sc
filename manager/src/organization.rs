use crate::config;

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait OrganizationModule: config::ConfigModule {
    #[only_owner]
    #[endpoint(initOrgModule)]
    fn init_organization_module(&self, org_contract: ManagedAddress) {
        self.org_contract_address().set(org_contract);
    }

    fn forward_payment_with_profit_share(&self, token_id: TokenIdentifier, amount: BigUint) {
        let org_contract = self.org_contract_address().get();

        self.org_contract_proxy(org_contract)
            .with_share_profits_endpoint()
            .add_esdt_token_transfer(token_id, 0, amount)
            .execute_on_dest_context_ignore_result();
    }

    #[storage_mapper("org:organization_contract_address")]
    fn org_contract_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn org_contract_proxy(&self, to: ManagedAddress) -> organization_proxy::Proxy<Self::Api>;
}

mod organization_proxy {
    elrond_wasm::imports!();

    #[elrond_wasm::proxy]
    pub trait OrganizationContractProxy {
        #[payable("*")]
        #[endpoint(withShareProfits)]
        fn with_share_profits_endpoint(&self);
    }
}
