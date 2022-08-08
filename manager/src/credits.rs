use crate::config;
use crate::dex;
use crate::events;
use crate::features;

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
pub trait CreditsModule: config::ConfigModule + features::FeaturesModule + dex::DexModule + events::EventsModule {
    #[payable("*")]
    #[endpoint(boost)]
    fn boost_endpoint(&self, entity_address: ManagedAddress) {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
        require!(payment.token_identifier == self.cost_token_id().get(), "invalid token");
        require!(payment.amount > 0, "amount can not be zero");

        self.boost(caller, entity_address, payment.amount)
    }

    #[payable("*")]
    #[endpoint(boostWithSwap)]
    fn boost_with_swap_endpoint(&self, entity: ManagedAddress, swap_contract_opt: OptionalValue<ManagedAddress>) {
        let (payment_token, payment_amount) = self.call_value().egld_or_single_fungible_esdt();
        let caller = self.blockchain().get_caller();
        let cost_token_id = self.cost_token_id().get();

        require!(payment_token.is_valid(), "invalid token");
        require!(payment_amount > 0, "amount can not be zero");
        require!(payment_token != cost_token_id, "no swap needed - call boost endpoint directly");

        let wegld = if payment_token.is_egld() {
            self.wrap_egld(payment_amount)
        } else {
            self.swap_tokens_to_wegld(payment_token.unwrap_esdt(), payment_amount, swap_contract_opt.into_option().unwrap())
        };

        let cost_payment = self.swap_wegld_to_cost_tokens(wegld.amount);

        self.boost(caller, entity, cost_payment.amount);
    }

    #[endpoint(registerExternalBoost)]
    fn register_external_boost_endpoint(&self, booster: ManagedAddress, entity: ManagedAddress, amount: BigUint) {
        let caller = self.blockchain().get_caller();
        let is_trusted_host = caller == self.trusted_host_address().get();
        let is_owner = caller == self.blockchain().get_owner_address();

        require!(is_trusted_host || is_owner, "not allowed");

        self.boost(booster, entity, amount);
    }

    #[view(getCredits)]
    fn get_credits_view(&self, entity_address: ManagedAddress) -> MultiValue2<BigUint, BigUint> {
        if self.credit_entries(&entity_address).is_empty() {
            return (BigUint::zero(), BigUint::zero()).into();
        }

        let entry = self.credit_entries(&entity_address).get();
        let available = self.calculate_available_credits(&entry);

        (available, entry.daily_cost).into()
    }

    fn boost(&self, booster: ManagedAddress, entity: ManagedAddress, amount: BigUint) {
        self.require_entity_exists(&entity);

        let mut entry = self.get_or_create_entry(&entity);
        entry.total_amount += &amount;
        entry.period_amount += &amount;

        self.credit_entries(&entity).set(entry);
        self.credit_total_deposits_amount().update(|current| *current += &amount);
        self.boost_event(booster, entity, amount);
    }

    fn recalculate_daily_cost(&self, entity_address: &ManagedAddress) {
        let mut entry = self.get_or_create_entry(&entity_address);
        let mut daily_cost = self.cost_base_daily_amount().get();

        entry.last_period_change = self.blockchain().get_block_timestamp();
        entry.period_amount = self.calculate_available_credits(&entry);

        for feature in self.features(&entity_address).iter() {
            daily_cost += self.cost_feature_daily_amount(&feature).get();
        }

        entry.daily_cost = daily_cost;

        self.credit_entries(&entity_address).set(entry);
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

    fn get_or_create_entry(&self, entity_address: &ManagedAddress) -> CreditEntry<Self::Api> {
        if self.credit_entries(&entity_address).is_empty() {
            CreditEntry {
                total_amount: BigUint::zero(),
                period_amount: BigUint::zero(),
                daily_cost: self.cost_base_daily_amount().get(),
                last_period_change: self.blockchain().get_block_timestamp(),
            }
        } else {
            self.credit_entries(&entity_address).get()
        }
    }

    #[storage_mapper("credits:entries")]
    fn credit_entries(&self, entity_address: &ManagedAddress) -> SingleValueMapper<CreditEntry<Self::Api>>;

    #[storage_mapper("credits:total_deposits")]
    fn credit_total_deposits_amount(&self) -> SingleValueMapper<BigUint>;
}
