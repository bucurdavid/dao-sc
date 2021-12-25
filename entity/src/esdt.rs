elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait EsdtModule {
    fn mint(&self, amount: &BigUint) -> SCResult<()> {
        let token_id = self.token_id().get();

        self.send().esdt_local_mint(&token_id, 0, amount);

        Ok(())
    }

    fn burn(&self, amount: &BigUint) -> SCResult<()> {
        let token_id = self.token_id().get();

        self.send().esdt_local_burn(&token_id, 0, amount);

        Ok(())
    }

    #[storage_mapper("token_id")]
    fn token_id(&self) -> SingleValueMapper<TokenIdentifier>;
}
