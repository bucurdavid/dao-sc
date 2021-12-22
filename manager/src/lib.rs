#![no_std]

elrond_wasm::imports!();

#[elrond_wasm::contract]
pub trait Manager {
    #[init]
    fn init(&self) {}
}
