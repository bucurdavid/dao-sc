use crate::config;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct CreditEntry<M: ManagedTypeApi> {
    pub total_amount: BigUint<M>,
    pub period_amount: BigUint<M>,
    pub daily_cost: BigUint<M>,
    pub last_period_change: u64,
}

#[elrond_wasm::module]
pub trait CreditsModule: config::ConfigModule {
    #[payable("*")]
    #[endpoint(boost)]
    fn boost_endpoint(&self, entity_token_id: TokenIdentifier) {
        let (payment_token_id, _, payment_amount) = self.call_value().payment_as_tuple();
        let caller = self.blockchain().get_caller();
        let entity_address = self.get_entity_address(&entity_token_id);

        require!(payment_token_id == self.cost_token_id().get(), "invalid token");
        require!(payment_amount >= self.cost_boost_min_amount().get(), "invalid amount");
        require!(entity_address == caller, "given token id does not belong to caller",);

        let mut entry = self.get_or_create_entry(&entity_address);
        entry.total_amount += &payment_amount;
        entry.period_amount += &payment_amount;

        self.credit_entries(&entity_address).set(entry);
        self.credit_total_deposits_amount().update(|current| *current += &payment_amount);
    }

    #[view(getAvailableCredits)]
    fn available_credits_view(&self, contract_address: ManagedAddress) -> BigUint {
        if self.credit_entries(&contract_address).is_empty() {
            return BigUint::zero();
        }

        let entry = self.credit_entries(&contract_address).get();

        self.calculate_available_credits(&entry)
    }

    #[view(getDailyCost)]
    fn daily_cost_view(&self, contract_address: ManagedAddress) -> BigUint {
        if self.credit_entries(&contract_address).is_empty() {
            return BigUint::zero();
        }

        let entry = self.credit_entries(&contract_address).get();

        entry.daily_cost
    }

    fn recalculate_daily_cost(&self, contract_address: &ManagedAddress, features: ManagedVec<ManagedBuffer>) {
        let mut entry = self.get_or_create_entry(&contract_address);
        let mut daily_cost = self.cost_base_daily_amount().get();

        entry.last_period_change = self.blockchain().get_block_timestamp();
        entry.period_amount = self.calculate_available_credits(&entry);

        for feature in features.into_iter() {
            daily_cost += self.cost_feature_daily_amount(&feature).get();
        }

        entry.daily_cost = daily_cost;

        self.credit_entries(&contract_address).set(entry);
    }

    fn calculate_available_credits(&self, entry: &CreditEntry<Self::Api>) -> BigUint {
        let seconds_in_period = self.blockchain().get_block_timestamp() - entry.last_period_change;
        let cost_per_second = &entry.daily_cost / &BigUint::from(86_400u64); // 1 day in seconds
        let credits_consumed = &cost_per_second * &BigUint::from(seconds_in_period);
        let available_credits = if &entry.period_amount > &credits_consumed {
            &entry.period_amount - &credits_consumed
        } else {
            BigUint::zero()
        };

        available_credits
    }

    fn get_or_create_entry(&self, contract_address: &ManagedAddress) -> CreditEntry<Self::Api> {
        if self.credit_entries(&contract_address).is_empty() {
            CreditEntry {
                total_amount: BigUint::zero(),
                period_amount: BigUint::zero(),
                daily_cost: self.cost_base_daily_amount().get(),
                last_period_change: self.blockchain().get_block_timestamp(),
            }
        } else {
            self.credit_entries(&contract_address).get()
        }
    }

    #[storage_mapper("credits:entries")]
    fn credit_entries(&self, contract_address: &ManagedAddress) -> SingleValueMapper<CreditEntry<Self::Api>>;

    #[storage_mapper("credits:total_deposits")]
    fn credit_total_deposits_amount(&self) -> SingleValueMapper<BigUint>;
}
