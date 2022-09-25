use crate::config;

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait DexModule: config::ConfigModule {
    #[only_owner]
    #[endpoint(initDexModule)]
    fn init_dex_module(&self, wegld_token_id: TokenIdentifier, cost_token_wegld_swap_contract: ManagedAddress, wrap_egld_contract: ManagedAddress) {
        self.wegld_token_id().set(wegld_token_id);
        self.cost_token_wegld_swap_contract().set(cost_token_wegld_swap_contract);
        self.wrap_egld_contract().set(wrap_egld_contract);
    }

    fn wrap_egld(&self, amount: BigUint) -> EsdtTokenPayment {
        let wegld_token_id = self.wegld_token_id().get();
        let wrap_egld_contract = self.wrap_egld_contract().get();

        self.wrap_egld_proxy(wrap_egld_contract)
            .wrap_egld()
            .with_egld_transfer(amount.clone())
            .execute_on_dest_context::<()>();

        EsdtTokenPayment::new(wegld_token_id, 0, amount)
    }

    fn swap_tokens_to_wegld(&self, payment_token: TokenIdentifier, payment_amount: BigUint, swap_contract: ManagedAddress) -> EsdtTokenPayment {
        let wegld_token_id = self.wegld_token_id().get();

        let swapped_wegld_legacy: dex_pair_proxy::LegacyDexPayment<Self::Api> = self
            .dex_pair_contract_proxy(swap_contract)
            .swap_tokens_fixed_input(wegld_token_id, BigUint::from(1u32))
            .add_esdt_token_transfer(payment_token, 0, payment_amount)
            .execute_on_dest_context();

        swapped_wegld_legacy.token_payment
    }

    fn swap_wegld_to_cost_tokens(&self, amount: BigUint) -> EsdtTokenPayment {
        let cost_token_id = self.cost_token_id().get();
        let wegld_token_id = self.wegld_token_id().get();
        let cost_token_wegld_swap_contract = self.cost_token_wegld_swap_contract().get();

        let swapped_cost_payment_legacy: dex_pair_proxy::LegacyDexPayment<Self::Api> = self
            .dex_pair_contract_proxy(cost_token_wegld_swap_contract)
            .swap_tokens_fixed_input(cost_token_id.clone(), BigUint::from(1u32))
            .add_esdt_token_transfer(wegld_token_id, 0, amount)
            .execute_on_dest_context();

        let swapped_cost_payment = swapped_cost_payment_legacy.token_payment;

        require!(swapped_cost_payment.token_identifier == cost_token_id, "swapped invalid cost token");

        swapped_cost_payment
    }

    #[storage_mapper("dex:wegld_token_id")]
    fn wegld_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("dex:cost_token_wegld_swap_contract")]
    fn cost_token_wegld_swap_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("dex:wrap_egld_contract")]
    fn wrap_egld_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn dex_pair_contract_proxy(&self, to: ManagedAddress) -> dex_pair_proxy::Proxy<Self::Api>;

    #[proxy]
    fn wrap_egld_proxy(&self, to: ManagedAddress) -> dex_wrap_proxy::Proxy<Self::Api>;
}

mod dex_pair_proxy {
    elrond_wasm::imports!();
    elrond_wasm::derive_imports!();

    #[derive(TopEncode, TopDecode, TypeAbi)]
    pub struct LegacyDexPayment<M: ManagedTypeApi> {
        pub token_type: u8,
        pub token_payment: SwapTokensFixedInputResultType<M>,
    }

    pub type SwapTokensFixedInputResultType<M> = EsdtTokenPayment<M>;

    #[elrond_wasm::proxy]
    pub trait DexRouterContractProxy {
        #[payable("*")]
        #[endpoint(swapTokensFixedInput)]
        fn swap_tokens_fixed_input(&self, token_out: TokenIdentifier, amount_out_min: BigUint) -> LegacyDexPayment<Self::Api>;
    }
}

mod dex_wrap_proxy {
    elrond_wasm::imports!();

    #[elrond_wasm::proxy]
    pub trait DexWrapContractProxy {
        #[payable("EGLD")]
        #[endpoint(wrapEgld)]
        fn wrap_egld(&self);
    }
}