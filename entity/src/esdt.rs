elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait EsdtModule {
    fn mint(&self, amount: &BigUint) {
        let token_id = self.token_id().get();

        self.send().esdt_local_mint(&token_id, 0, amount);
    }

    fn burn(&self, amount: &BigUint) {
        let token_id = self.token_id().get();

        self.send().esdt_local_burn(&token_id, 0, amount);
    }

    #[storage_mapper("token_id")]
    fn token_id(&self) -> SingleValueMapper<TokenIdentifier>;
}
