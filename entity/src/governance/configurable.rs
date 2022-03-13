elrond_wasm::imports!();

use super::storage;

#[elrond_wasm::module]
pub trait GovConfigurableModule: storage::GovStorageModule {
    fn init_governance_module(&self, governance_token_id: &TokenIdentifier, initial_tokens: &BigUint) {
        require!(governance_token_id.is_valid_esdt_identifier(), "invalid edst");

        let initial_quorum = initial_tokens / &BigUint::from(20u32); // 5% of initial tokens
        let initial_min_tokens_for_proposing = initial_tokens / &BigUint::from(100u32); // 1% of initial tokens
        let initial_voting_period_hours = 72u32; // 3 days

        self.governance_token_id().set_if_empty(&governance_token_id);
        self.try_change_quorum(BigUint::from(initial_quorum));
        self.try_change_min_token_balance_for_proposing(BigUint::from(initial_min_tokens_for_proposing));
        self.try_change_voting_period_in_hours(initial_voting_period_hours);
    }

    #[endpoint(changeQuorum)]
    fn change_quorum(&self, new_value: BigUint) {
        self.require_caller_self();
        self.try_change_quorum(new_value);
    }

    #[endpoint(changeMinTokenBalanceForProposing)]
    fn change_min_token_balance_for_proposing(&self, new_value: BigUint) {
        self.require_caller_self();
        self.try_change_min_token_balance_for_proposing(new_value);
    }

    #[endpoint(changeVotingPeriodInBlocks)]
    fn change_voting_period_in_hours(&self, new_value: u32) {
        self.require_caller_self();
        self.try_change_voting_period_in_hours(new_value);
    }

    fn require_caller_self(&self) {
        let caller = self.blockchain().get_caller();
        let sc_address = self.blockchain().get_sc_address();

        require!(caller == sc_address, "action not allowed");
    }

    fn try_change_quorum(&self, new_value: BigUint) {
        require!(new_value != 0, "invalid quorum");
        self.quorum().set(&new_value);
    }

    fn try_change_min_token_balance_for_proposing(&self, new_value: BigUint) {
        require!(new_value != 0, "min token balance for proposing can not be 0");
        self.min_token_balance_for_proposing().set(&new_value);
    }

    fn try_change_voting_period_in_hours(&self, new_value: u32) {
        require!(new_value != 0, "voting period (in hours) can not be 0");
        self.voting_period_in_hours().set(&new_value);
    }
}
