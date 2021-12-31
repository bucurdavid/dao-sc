elrond_wasm::imports!();

use super::storage;

const DEFAULT_VOTING_PERIOD: u64 = 100_800; // ~7 days in blocks (assuming 6s blocks)
const DEFAULT_VOTING_DELAY: u64 = 1; // 1 block
const DEFAULT_VOTING_LOCKTIME: u64 = 28_800; // 2 days
const DEFAULT_VOTING_MAX_ACTIONS: usize = 10;

#[elrond_wasm::module]
pub trait GovConfigurableModule: storage::GovStorageModule {
    #[only_owner]
    #[endpoint(initGovernanceModule)]
    fn init_governance_module(&self, governance_token_id: TokenIdentifier, quorum: BigUint, min_token_balance_for_proposal: BigUint) -> SCResult<()> {
        require!(governance_token_id.is_valid_esdt_identifier(), "invalid edst");

        self.governance_token_id().set_if_empty(&governance_token_id);

        self.try_change_quorum(quorum)?;
        self.try_change_min_token_balance_for_proposing(min_token_balance_for_proposal)?;
        self.try_change_max_actions_per_proposal(DEFAULT_VOTING_MAX_ACTIONS)?;
        self.try_change_voting_delay_in_blocks(DEFAULT_VOTING_DELAY)?;
        self.try_change_voting_period_in_blocks(DEFAULT_VOTING_PERIOD)?;
        self.try_change_lock_time_after_voting_ends_in_blocks(DEFAULT_VOTING_LOCKTIME)?;

        Ok(())
    }

    #[endpoint(changeQuorum)]
    fn change_quorum(&self, new_value: BigUint) -> SCResult<()> {
        self.require_caller_self()?;
        self.try_change_quorum(new_value)?;

        Ok(())
    }

    #[endpoint(changeMinTokenBalanceForProposing)]
    fn change_min_token_balance_for_proposing(&self, new_value: BigUint) -> SCResult<()> {
        self.require_caller_self()?;
        self.try_change_min_token_balance_for_proposing(new_value)?;

        Ok(())
    }

    #[endpoint(changeMaxActionsPerProposal)]
    fn change_max_actions_per_proposal(&self, new_value: usize) -> SCResult<()> {
        self.require_caller_self()?;
        self.try_change_max_actions_per_proposal(new_value)?;

        Ok(())
    }

    #[endpoint(changeVotingDelayInBlocks)]
    fn change_voting_delay_in_blocks(&self, new_value: u64) -> SCResult<()> {
        self.require_caller_self()?;
        self.try_change_voting_delay_in_blocks(new_value)?;

        Ok(())
    }

    #[endpoint(changeVotingPeriodInBlocks)]
    fn change_voting_period_in_blocks(&self, new_value: u64) -> SCResult<()> {
        self.require_caller_self()?;
        self.try_change_voting_period_in_blocks(new_value)?;

        Ok(())
    }

    #[endpoint(changeLockTimeAfterVotingEndsInBlocks)]
    fn change_lock_time_after_voting_ends_in_blocks(&self, new_value: u64) -> SCResult<()> {
        self.require_caller_self()?;
        self.try_change_lock_time_after_voting_ends_in_blocks(new_value)?;

        Ok(())
    }

    fn require_caller_self(&self) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        let sc_address = self.blockchain().get_sc_address();

        require!(caller == sc_address, "action not allowed");

        Ok(())
    }

    fn try_change_quorum(&self, new_value: BigUint) -> SCResult<()> {
        require!(new_value != 0, "invalid quorum");
        self.quorum().set(&new_value);

        Ok(())
    }

    fn try_change_min_token_balance_for_proposing(&self, new_value: BigUint) -> SCResult<()> {
        require!(new_value != 0, "min token balance for proposing can't be set to 0");
        self.min_token_balance_for_proposing().set(&new_value);

        Ok(())
    }

    fn try_change_max_actions_per_proposal(&self, new_value: usize) -> SCResult<()> {
        require!(new_value != 0, "max actions per proposal can't be set to 0");
        self.max_actions_per_proposal().set(&new_value);

        Ok(())
    }

    fn try_change_voting_delay_in_blocks(&self, new_value: u64) -> SCResult<()> {
        require!(new_value != 0, "voting delay in blocks can't be set to 0");
        self.voting_delay_in_blocks().set(&new_value);

        Ok(())
    }

    fn try_change_voting_period_in_blocks(&self, new_value: u64) -> SCResult<()> {
        require!(new_value != 0, "voting period (in blocks) can't be set to 0");
        self.voting_period_in_blocks().set(&new_value);

        Ok(())
    }

    fn try_change_lock_time_after_voting_ends_in_blocks(&self, new_value: u64) -> SCResult<()> {
        require!(new_value != 0, "lock time after voting ends (in blocks) can't be set to 0");
        self.lock_time_after_voting_ends_in_blocks().set(&new_value);

        Ok(())
    }
}
