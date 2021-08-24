use near_sdk::json_types::{U64, U128};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{ext_contract, log, near_bindgen, env, PromiseResult};
use near_sdk::serde::{Deserialize, Serialize};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolInfo {
  pub total_staked: U128,
  pub reserve: U128,
  pub prize: U128,
  pub next_prize_tmstmp: U64,
  pub withdraw_ready: bool
}

#[ext_contract(this_contract)]
trait Callbacks {
  fn exchange_tokens_for_tickets_callback(&mut self, user: AccountId, tokens: Balance, tickets:Balance) -> bool;
  fn exchange_near_for_tokens_callback(&mut self, user: AccountId, tokens: Balance, tickets: Balance) -> bool ;
  fn cache_pool_party_reserve_callback(&mut self);  
}

// Pool Party interface, so we can do async calls
#[ext_contract(poolparty_contract)]
trait PoolParty {
    #[payable]
    fn deposit_and_stake(&mut self) -> bool;
    fn get_pool_info(&self) -> PoolInfo;
    fn give_from_reserve(&self, to: AccountId, amount:U128);
}

// Aux functions to interact with pool party

pub fn did_promise_succeded() -> bool {
  if env::promise_results_count() != 1 {
    log!("Expected a result on the callback");
    return false;
  }

  match env::promise_result(0) {
      PromiseResult::Successful(_) => true,
      _ => false,
  }
}
