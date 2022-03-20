elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait ConfigModule {
    #[view(getEntityTemplateAddress)]
    #[storage_mapper("entity_templ_address")]
    fn entity_templ_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("vote_nft_token_id")]
    fn vote_nft_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("cost_token")]
    fn cost_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("cost_creation_amount")]
    fn cost_creation_amount(&self) -> SingleValueMapper<BigUint>;
}
