use crate::config;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait OrganizationModule: config::ConfigModule {
    #[only_owner]
    #[endpoint(initOrgModule)]
    fn init_organization_module(&self, org_contract: ManagedAddress) {
        self.org_contract_address().set(org_contract);
    }

    #[only_owner]
    #[endpoint(forwardCostTokensToOrg)]
    fn forward_cost_tokens_to_org(&self) {
        require!(!self.org_contract_address().is_empty(), "org address must be configured");
        let cost_token_id = self.cost_token_id().get();
        let balance = self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(cost_token_id.clone()), 0);

        self.forward_payment_to_org(EsdtTokenPayment::new(cost_token_id, 0, balance));
    }

    fn forward_payment_to_org(&self, payment: EsdtTokenPayment) {
        if self.org_contract_address().is_empty() {
            return;
        }

        self.org_contract_proxy(self.org_contract_address().get())
            .distribute_endpoint()
            .with_esdt_transfer(payment)
            .execute_on_dest_context::<()>();
    }

    #[storage_mapper("org:organization_contract_address")]
    fn org_contract_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn org_contract_proxy(&self, to: ManagedAddress) -> organization_proxy::Proxy<Self::Api>;
}

mod organization_proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait OrganizationContractProxy {
        #[payable("*")]
        #[endpoint(distribute)]
        fn distribute_endpoint(&self);
    }
}
