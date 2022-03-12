elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub type ActionAsMultiArg<M> =
    MultiValue7<u64, ManagedAddress<M>, TokenIdentifier<M>, u64, BigUint<M>, ManagedBuffer<M>, ManagedVec<M, ManagedBuffer<M>>>;

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq)]
pub enum ProposalStatus {
    None,
    Active,
    Defeated,
    Succeeded,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem)]
pub struct Action<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub endpoint: ManagedBuffer<M>,
    pub arguments: ManagedVec<M, ManagedBuffer<M>>,
    pub gas_limit: u64,
    pub token_id: TokenIdentifier<M>,
    pub token_nonce: u64,
    pub amount: BigUint<M>,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct Proposal<M: ManagedTypeApi> {
    pub proposer: ManagedAddress<M>,
    pub title: ManagedBuffer<M>,
    pub description: ManagedBuffer<M>,
    pub actions: ManagedVec<M, Action<M>>,
}
