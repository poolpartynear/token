/*! Pool Token */
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{env, log, PromiseResult, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, Promise, PromiseOrValue};
use uint::construct_uint;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

pub mod external;


pub use crate::external::{this_contract, poolparty_contract, PoolInfo};

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    pool_party_reserve: u128,
    pool_party_next_raffle: u64,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";
const POOL_PARTY_ACCOUNT: &str = "test-account-1629666757916-4879641";

const NO_DEPOSIT: Balance = 0;
const TGAS: Gas = 1_000_000_000_000;
const TIME_THRESHOLD: u64 = 300_000_000_000; // 5 minutes

#[near_bindgen]
impl Contract {

    #[init]
    pub fn new( ) -> Self {
        assert!(!env::state_exists(), "Already initialized");

        let owner_id: AccountId = String::from("test-account-1629671080959-5153795");

        let metadata = FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: "Pool Party fungible token".to_string(),
            symbol: "$POOL".to_string(),
            icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
            reference: None,
            reference_hash: None,
            decimals: 8,
        };

        metadata.assert_valid();
        let mut this = Self {
            pool_party_reserve: 0,
            pool_party_next_raffle: 0,
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
        };

        // Register this contract as a user so it can receive and give tokens
        this.token.internal_register_account(&env::current_account_id()); 

        // HARCODE total supply and give it to the owner_id
        let total_supply: Balance = 10_000_000;
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply);

        this
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }

    // We cache the reserve of Pool Party since it gets updated once per day at max
    pub fn cache_pool_party_reserve(&mut self) -> Promise {
        poolparty_contract::get_pool_info(
            &POOL_PARTY_ACCOUNT,
            NO_DEPOSIT,
            20*TGAS
        ).then(this_contract::cache_pool_party_reserve_callback(
            &env::current_account_id(),
            NO_DEPOSIT,
            5*TGAS,
        ))
    }

    #[private]
    pub fn cache_pool_party_reserve_callback(&mut self) -> bool {
        if !external::did_promise_succeded() {
            log!("No result found on callback");
            return false;
        }

        // Get response, return false if failed
        let pool_info: PoolInfo = match env::promise_result(0) {
            PromiseResult::Successful(value) => near_sdk::serde_json::from_slice::<PoolInfo>(&value).unwrap(),
            _ => { log!("Getting info from Pool Party failed"); return false; },
        };

        let next_raffle = u64::from(pool_info.next_prize_tmstmp);

        if next_raffle <= self.pool_party_next_raffle {
            log!("No need to update");
            return true
        }

        self.pool_party_reserve = u128::from(pool_info.reserve);
        self.pool_party_next_raffle = next_raffle;

        log!("Reserve: {}. Next update: {}", self.pool_party_reserve, next_raffle);
        true
    }

    // Assert we are at least T min. away from the raffle, and that we didn't used the reserve
    fn panic_if_close_to_raffle(&mut self) {
        assert!(env::block_timestamp() < self.pool_party_next_raffle - TIME_THRESHOLD, 
                "Cannot exchange right before the raffle. Wait for the raffle, or update the cache");
    }

    // Exchange $POOL tokens for tickets in the reserve of Pool Party
    pub fn exchange_tokens_for_tickets(&mut self, amount_tokens: U128) -> Promise {
        assert!(env::prepaid_gas() >= 120 * TGAS, "This method requires at least 120 TGAS to run");

        // Assert we are at least T min. away from the raffle, to ensure the cached reserve is valid
        self.panic_if_close_to_raffle();

        // compute how many tickets correspond to the user
        let amount_tokens_u128 = u128::from(amount_tokens);
        let tokens_own_by_contract = self.token.internal_unwrap_balance_of(&env::current_account_id());
        let amount_tickets: U256 = U256::from(self.pool_party_reserve * amount_tokens_u128) / U256::from(self.token.total_supply - tokens_own_by_contract);
        let amount_tickets_u128 = amount_tickets.as_u128();

        // Remove them from the cached reserve
        self.pool_party_reserve -= amount_tickets_u128;

        // Transfer the tokens from the user to this contract
        let user: AccountId = env::predecessor_account_id();
        let this: AccountId = env::current_account_id();
        self.token.internal_transfer(&user, &this, amount_tokens_u128, None); //checks balance

        // ask to transfer tickets to the user
        poolparty_contract::give_from_reserve(
            user.clone(),
            U128::from(amount_tickets_u128),
            &POOL_PARTY_ACCOUNT,
            NO_DEPOSIT,
            120*TGAS
        ).then(this_contract::exchange_tokens_for_tickets_callback(
            user.clone(),
            amount_tokens_u128,
            amount_tickets_u128,
            &env::current_account_id(),
            NO_DEPOSIT,
            50*TGAS,
        ))
    }

    
    #[private] // checks that caller_id == this contract
    pub fn exchange_tokens_for_tickets_callback(&mut self, user: AccountId, tokens: Balance, tickets:Balance) -> bool {
        if !external::did_promise_succeded(){
            log!("Failed, returning tokens to {}", &user);
            let this: AccountId = env::current_account_id();
            self.token.internal_transfer(&this, &user, tokens, None);
            self.pool_party_reserve += tickets;
            return false
        }

        true
    }

    #[payable]
    pub fn exchange_near_for_tokens(&mut self) -> Promise {
        assert!(env::prepaid_gas() >= 30 * TGAS, "This method requires at least 30 TGAS to run");

        // Assert we are at least T min. away from the raffle, to ensure the cached reserve is valid
        self.panic_if_close_to_raffle();

        let near_amount = env::attached_deposit();

        // price_per_token = self.pool_party_reserve / ( self.token.total_supply - amount_tokens_already_bought);
        // token_amount = near_amount / price_per_token
        let tokens_own_by_contract = self.token.internal_unwrap_balance_of(&env::current_account_id());
        let tot_minus_ours = self.token.total_supply - tokens_own_by_contract;
        log!("total - ours: {}, PP reserve: {}", tot_minus_ours, self.pool_party_reserve);
        let token_amount = (U256::from(near_amount * tot_minus_ours) / U256::from(self.pool_party_reserve)).as_u128();

        log!("Exchanging {} N for {} tokens", near_amount, token_amount);

        // Check if we have enought tokens to sell
        assert!(token_amount <= tokens_own_by_contract, "We do not have enough tokens to sell");

        // deposit the money in pool party
        poolparty_contract::deposit_and_stake(
            &POOL_PARTY_ACCOUNT,
            near_amount,
            190*TGAS
        ).then(this_contract::exchange_near_for_tokens_callback(
            env::predecessor_account_id(),
            token_amount,
            near_amount,
            &env::current_account_id(),
            NO_DEPOSIT,
            20*TGAS,
        ))
    }

    #[private]
    pub fn exchange_near_for_tokens_callback(&mut self, user: AccountId, tokens: Balance, tickets: Balance) -> bool {
        if external::did_promise_succeded(){
            // Succeeded in staking NEARs, transfer tokens to the user
            let this = env::current_account_id();
            self.token.internal_transfer(&this, &user, tokens, None);
            self.pool_party_reserve += tickets;
            return true
        }
        // Failed to stake nears, panic so the user gets back the money
        panic!("Failed to stake tokens in Pool Party")
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 10_000_000;

    fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new();
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new();
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(contract.ft_balance_of(accounts(2)).0, (TOTAL_SUPPLY - transfer_amount));
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }

    // Test you cannot immediately call any method
}
