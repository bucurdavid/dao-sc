#![no_std]

elrond_wasm::imports!();

#[elrond_wasm::contract]
pub trait Entity {
    #[init]
    fn init(&self) {}
}
