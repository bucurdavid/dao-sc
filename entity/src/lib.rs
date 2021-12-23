#![no_std]

elrond_wasm::imports!();

mod esdt;
mod features;

#[elrond_wasm::contract]
pub trait Entity: features::FeaturesModule + esdt::EsdtModule {
    #[init]
    fn init(&self) {}
}
