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

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,<svg viewBox='0 0 17.81 17.954' xmlns='http://www.w3.org/2000/svg' <g transform='translate(-8.4817 -206.38)'><g transform='translate(-131.99 168.02)'><g transform='matrix(.16504 0 0 .16504 -29.513 -136.23)'><path class='st26' d='m1136.9 1102.3 0.1087 0.7845c-0.2986-1.3087-0.5707-4.7987-3.1762-11.482l0.2905 0.8026c-3.27-7.517-6.2674-12.946-12.124-18.703 0.049 0.048 0.102 0.092 0.1496 0.1413-3.5617-3.4055-5.5545-5.1898-10.808-8.445l0.2239 0.1315c-4.1656-1.8953-11.893-7.4158-26.924-7.6325h0.4904c-0.6298 0.048-8.6208-0.5124-18.364 2.798l0.073-0.027c-0.1044 0.038-2.2771 0.8257-2.1729 0.7874-10.814 4.1862-14.868 8.7213-20.211 13.629-8.9441 10.017-11.324 18.883-11.289 18.783-8.8448 24.792 1.5598 56.016 31.729 68.136 0.9618 0.086 27.838 12.916 55.54-10.797 0.1125-0.193 21.713-18.833 16.465-48.905zm-1.4388-5.8871c0.099 0.3423 0.1906 0.681 0.2812 1.0191-0.1262-0.4389-0.1904-0.7088-0.2812-1.0191zm-0.1259-0.4192c-0.1166-0.3876-0.2407-0.7611-0.3629-1.1382-0.4448-1.4365-0.038-0.2095 0.3629 1.1382z' fill='#f5dc13'/><path class='st27' d='m1080.2 1058c11.006-0.8868 22.481 2.2354 31.087 7.37 5.3935 3.218 8.1741 5.8922 11.127 8.7611 18.988 19.497-16.91 71.924-54.451 79.726 37.323-7.7561 73.519-60.147 54.451-79.726 46.398 48.052-21.17 125.61-74.892 77.178-12.465-11.237-19.094-29.059-17.144-45.274 3.183-26.472 23.325-45.899 49.822-48.034z' fill='none'/><path class='st28' d='m1065.6 1152.7c0.3903-0.5048 1.262-0.5531 1.8918-0.4665-4.8334-18.374-2.0456-50.572 6.4826-73.168 5.378-14.249 11.793-21.379 17.218-20.644-19.109-2.1578-37.645 4.7318-49.881 20.392-17.098 22.196-5.4359 57.904 24.289 73.886z' fill='#f47638'/><path class='st29' d='m1065.9 1154.2c-0.5161-0.5629-0.6039-1.1274-0.3034-1.5162-30.046-16.155-41.219-51.908-24.289-73.886-2.567 3.6136-4.7987 6.4836-7.7446 13.899-4.4474 13.367-4.6159 23.266-0.7547 35.93 8.9613 24.109 19.954 26.857 33.092 25.574z' fill='#4599d4'/><path class='st26' d='m1068.1 1155.4c-0.799-0.1834-1.6486-0.6053-2.1909-1.1971-6.9651 0.587-14.966 0.6354-21.772-6.2624 0.3198 0.3192 2.4555 2.7554 5.8885 5.5635 11.951 9.2159 22.86 10.883 21.858 10.643-1.883-0.4264-3.2279-3.2916-3.7838-8.7469z' fill='#f5dc13'/><path class='st28' d='m1070.2 1155.2c-0.5285 0.4156-1.4437 0.3814-2.109 0.2288 0.5559 5.4553 1.9008 8.3205 3.7838 8.7469 12.558 2.5892 23.464 1.3104 34.303-3.4734 7.7317-3.8474 11.164-6.8024 11.064-6.7351-12.008 9.4899-29.964 10.486-47.041 1.2328z' fill='#f47638'/><path class='st29' d='m1070.3 1153.7c0.3845 0.5592 0.3469 1.0924-0.073 1.4226 17.077 9.2532 35.033 8.2571 47.041-1.2328 11.643-9.9233 17.208-19.989 19.7-33.313 2.0478-15.859-0.8387-27.309-8.6075-39.221 15.97 26.79-16.587 67.358-58.062 72.345z' fill='#4599d4'/><path class='st27' d='m1045.3 1149.2c-18.997-20.032-18.964-47.757-7.1843-65.92 5.8835-9.0721 13.291-16.028 23.366-20.561 20.051-9.0218 44.13-5.6545 60.873 11.406-4.72-4.8466-12.513-6.7474-22.606-4.6498 10.094-2.0976 17.886-0.1968 22.606 4.6498 4.5561 4.9653 5.1762 6.2052 6.5569 8.1873 7.5831 10.886 11.056 28.949 7.0943 42.437-3.8054 12.958-8.2128 20.094-18.735 29.073-18.792 16.036-52.432 15.983-71.971-4.6216z' fill='none'/><path class='st28' d='m1041.3 1078.8c13.148-15.744 29.723-22.726 49.881-20.392 1.5507 0.2102 3.0205 1.0632 4.3621 2.5764-4.2563-0.5851-10.145-0.2577-12.782 3.1526-16.704-2.5614-32.381 2.8754-41.462 14.663z' fill='#f47638'/><path class='st29' d='m1044.1 1147.9c-18.508-21.607-18.22-48.666-2.8207-69.14 9.0804-11.788 24.757-17.225 41.462-14.663-2.0305 2.6266-1.4376 6.4406 2.049 10.244-38.294 19.167-54.652 57.101-40.69 73.559z' fill='#4599d4'/><path class='st26' d='m1071.9 1164.1c-10.057-2.5925-18.246-6.4741-26.516-14.9l-1.2303-1.3061c-15.84-20.122 9.4946-58.498 40.69-73.559 0.1295 0.1414 0.2632 0.2827 0.4008 0.4241 3.5639 3.6595 8.9953 6.4225 14.403 7.6634-2.6311 34.879-18.8 83.704-27.747 81.678z' fill='#f5dc13'/><path class='st28' d='m1117.2 1153.9c-14.743 10.223-27.307 14.01-45.366 10.208 9.303 2.1066 25.227-48.275 27.747-81.678 3.4524 0.7921 6.895 0.9641 9.8358 0.3529 1.8443-0.3832 3.3206-1.0389 4.4135-1.8985 21.19 22.725 23.718 56.934 3.37 73.015z' fill='#f47638'/><path class='st29' d='m1128.3 1081.4c17.14 26.972 9.1903 56.496-11.093 72.534 20.934-16.544 17.21-50.945-3.37-73.015 2.8355-2.2305 3.0897-5.8338 0.4919-9.6121 3.4009 1.2882 9.7311 3.6862 13.971 10.093z' fill='#4599d4'/><ellipse class='st1' transform='matrix(.34241 -.93955 .93955 .34241 -284.5 1737.5)' cx='1099' cy='1072' rx='10.244' ry='18.207' fill='#f7f7fb'/></g></g></g></svg>";
const POOL_PARTY_ACCOUNT: &str = "pool.pooltest.testnet";

const NO_DEPOSIT: Balance = 0;
const TGAS: Gas = 1_000_000_000_000;
const TIME_THRESHOLD: u64 = 300_000_000_000; // 5 minutes

#[near_bindgen]
impl Contract {

    #[init]
    pub fn new( ) -> Self {
        assert!(!env::state_exists(), "Already initialized");

        let owner_id: AccountId = String::from("gagdiez.testnet");

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
        self.token.internal_transfer(&user, &this, amount_tokens_u128, None);

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

        // Failed to stake nears, send money back to the user
        Promise::new(user).transfer(tickets);
        return false
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
