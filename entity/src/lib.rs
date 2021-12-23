#![no_std]

elrond_wasm::imports!();

mod features;

#[elrond_wasm::contract]
pub trait Entity: features::FeaturesModule {
    #[init]
    fn init(&self) {}
}
