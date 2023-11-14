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
    #[endpoint(stageContract)]
    fn stage_contract_endpoint(&self, address: ManagedAddress, code: ManagedBuffer) {
        // TODO: guard caller self or has permission like having built-in developer role

        self.contract_stage(&address).set(&code);
    }

    #[endpoint(activateContract)]
    fn activate_contract_endpoint(&self, address: ManagedAddress, gas: u64, egld_value: BigUint, code_metadata: CodeMetadata, args: MultiValueEncoded<ManagedBuffer>) {
        self.require_caller_self();
        require!(!self.contract_stage(&address).is_empty(), "contract not staged");

        let args_buffer = args.to_arg_buffer();
        let code = self.contract_stage(&address).take();

        if address.is_zero() {
            self.send_raw().deploy_contract(gas, &egld_value, &code, code_metadata, &args_buffer);
        } else {
            self.send_raw().upgrade_contract(&address, gas, &egld_value, &code, code_metadata, &args_buffer);
        }
    }

    #[storage_mapper("contract:stage")]
    fn contract_stage(&self, address: &ManagedAddress) -> SingleValueMapper<ManagedBuffer>;
}
