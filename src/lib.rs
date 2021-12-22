#![no_std]

elrond_wasm::imports!();

#[elrond_wasm::contract]
pub trait Fellowships {
    #[init]
    fn init(&self) {}
}
