elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct Proposal<M: ManagedTypeApi> {
    pub id: u64,
    pub proposer: ManagedAddress<M>,
    pub title: ManagedBuffer<M>,
    pub description: ManagedBuffer<M>,
    pub starts_at: u64,
    pub ends_at: u64,
    pub was_executed: bool,
    pub actions: ManagedVec<M, Action<M>>,
    pub votes_for: BigUint<M>,
    pub votes_against: BigUint<M>,
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

pub type ActionAsMultiArg<M> =
    MultiValue7<u64, ManagedAddress<M>, TokenIdentifier<M>, u64, BigUint<M>, ManagedBuffer<M>, ManagedVec<M, ManagedBuffer<M>>>;

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq)]
pub enum ProposalStatus {
    None,
    Active,
    Defeated,
    Succeeded,
}

#[elrond_wasm::module]
pub trait ProposalModule {
    fn execute_proposal(&self, proposal: &Proposal<Self::Api>) {
        for action in proposal.actions.iter() {
            let mut call = self
                .send()
                .contract_call::<()>(action.address, action.endpoint)
                .with_gas_limit(action.gas_limit);

            if action.amount > 0 {
                call = if action.token_id == TokenIdentifier::egld() {
                    call.with_egld_transfer(action.amount)
                } else {
                    call.add_token_transfer(action.token_id, action.token_nonce, action.amount)
                }
            }

            call.transfer_execute()
        }
    }
}
