multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait EventsModule {
    #[event("boost")]
    fn boost_event(
        &self,
        #[indexed] booster: ManagedAddress,
        #[indexed] entity: ManagedAddress,
        #[indexed] amount: BigUint,
        #[indexed] virtual_amount: BigUint,
        #[indexed] bonus_factor: u8,
    );
}
