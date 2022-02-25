elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait CostModule {
    fn init_cost_module(&self, currency_token: TokenIdentifier, creation_cost: BigUint) {
        self.cost_token_id().set_if_empty(&currency_token);
        self.cost_creation_amount().set_if_empty(&creation_cost);
    }

    fn burn_entity_creation_cost_tokens(&self, cost_token_id: TokenIdentifier, amount: BigUint) {
        require!(cost_token_id == self.cost_token_id().get(), "invalid cost token");
        require!(amount >= self.cost_creation_amount().get(), "invalid cost amount");

        self.send().esdt_local_burn(&cost_token_id, 0, &amount);
    }

    #[storage_mapper("currency_token")]
    fn cost_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("cost_creation_amount")]
    fn cost_creation_amount(&self) -> SingleValueMapper<BigUint>;
}
