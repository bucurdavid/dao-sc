elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait EventsModule {
    #[event("boost")]
    fn boost_event(&self, #[indexed] booster: ManagedAddress, #[indexed] entity: ManagedAddress, #[indexed] virtual_amount: BigUint, #[indexed] bonus_factor: u8);
}
