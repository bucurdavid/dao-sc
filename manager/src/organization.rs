use crate::config;

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait OrganizationModule: config::ConfigModule {
    #[only_owner]
    #[endpoint(initOrgModule)]
    fn init_organization_module(&self, org_contract: ManagedAddress) {
        self.org_contract_address().set(org_contract);
    }

    #[only_owner]
    #[endpoint(forwardCostTokensToOrg)]
    fn forward_cost_tokens_to_org(&self) {
        require!(!self.org_contract_address().is_empty(), "org address must be conigured");
        let cost_token_id = self.cost_token_id().get();
        let balance = self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(cost_token_id.clone()), 0);

        self.forward_payment_to_org(EsdtTokenPayment::new(cost_token_id, 0, balance));
    }

    fn forward_payment_to_org(&self, payment: EsdtTokenPayment) {
        if self.org_contract_address().is_empty() {
            return;
        }

        self.org_contract_proxy(self.org_contract_address().get())
            .deposit_endpoint()
            .add_esdt_token_transfer(payment.token_identifier, payment.token_nonce, payment.amount)
            .execute_on_dest_context::<()>();
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
        #[endpoint(deposit)]
        fn deposit_endpoint(&self);
    }
}
