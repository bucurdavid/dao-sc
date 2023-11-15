multiversx_sc::imports!();

use crate::config;
use crate::governance;
use crate::governance::events;
use crate::permission;
use crate::plug;

#[multiversx_sc::module]
pub trait ContractModule:
    config::ConfigModule + governance::proposal::ProposalModule + permission::PermissionModule + events::GovEventsModule + plug::PlugModule
{
    #[endpoint(lockContractStage)]
    fn lock_contract_stage_endpoint(&self, address: ManagedAddress) {
        self.require_caller_self();

        let locked = self.stage_lock(&address).get();
        require!(!locked, "contract stage is locked already");

        self.stage_lock(&address).set(true);
    }

    #[endpoint(unlockContractStage)]
    fn unlock_contract_stage_endpoint(&self, address: ManagedAddress) {
        self.require_caller_self();

        let locked = self.stage_lock(&address).get();
        require!(locked, "contract stage is unlocked already");

        self.stage_lock(&address).clear();
        self.stage(&address).clear();
    }

    #[endpoint(stageContract)]
    fn stage_contract_endpoint(&self, address: ManagedAddress, code: ManagedBuffer) {
        // TODO: guard caller self or has permission like having built-in developer role

        self.stage_contract(&address, &code);
    }

    #[endpoint(stageContractAndPropose)]
    fn stage_contract_and_propose_endpoint(
        &self,
        address: ManagedAddress,
        code: ManagedBuffer,
        trusted_host_id: ManagedBuffer,
        content_hash: ManagedBuffer,
        content_sig: ManagedBuffer,
        actions_hash: ManagedBuffer,
        permissions: MultiValueManagedVec<ManagedBuffer>,
    ) {
        let caller = self.blockchain().get_caller();
        // TODO: guard caller self or has permission like having built-in developer role

        self.stage_contract(&address, &code);

        self.create_proposal(
            caller,
            trusted_host_id,
            content_hash,
            content_sig,
            actions_hash,
            0,
            BigUint::zero(),
            permissions.into_vec(),
        );
    }

    #[endpoint(activateContract)]
    fn activate_contract_endpoint(&self, address: ManagedAddress, gas: u64, egld_value: BigUint, code_metadata: CodeMetadata, args: MultiValueEncoded<ManagedBuffer>) {
        self.require_caller_self();
        require!(!self.stage(&address).is_empty(), "contract not staged");

        let args_buffer = args.to_arg_buffer();
        let code = self.stage(&address).take();

        if address.is_zero() {
            self.send_raw().deploy_contract(gas, &egld_value, &code, code_metadata, &args_buffer);
        } else {
            self.send_raw().upgrade_contract(&address, gas, &egld_value, &code, code_metadata, &args_buffer);
        }

        self.stage_lock(&address).clear();
    }

    fn stage_contract(&self, address: &ManagedAddress, code: &ManagedBuffer) {
        let locked = self.stage_lock(&address).get();
        require!(locked == false, "contract stage is locked");

        self.stage(&address).set(code);
        self.stage_lock(&address).set(true);
    }

    #[storage_mapper("contract:stage")]
    fn stage(&self, address: &ManagedAddress) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("contract:stage_lock")]
    fn stage_lock(&self, address: &ManagedAddress) -> SingleValueMapper<bool>;
}
