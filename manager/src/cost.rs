elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait CostModule {
    fn init_cost_module(&self, currency_token: TokenIdentifier, creation_cost: BigUint) {
        self.cost_token_id().set_if_empty(&currency_token);
        self.cost_creation_amount().set_if_empty(&creation_cost);
    }

    fn burn_cost_tokens(&self, &cost_token_id: &TokenIdentifier, amount: &BigUint) {
        let expected_cost_token_id = self.cost_token_id().get();
        require!(cost_token_id == expected_cost_token_id, "invalid cost token");
        self.send().esdt_local_burn(&expected_cost_token_id, 0, &amount);
    }

    #[storage_mapper("currency_token")]
    fn cost_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("cost_creation_amount")]
    fn cost_creation_amount(&self) -> SingleValueMapper<BigUint>;
}
