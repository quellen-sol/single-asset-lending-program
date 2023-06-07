use anchor_spl::token::TokenAccount;

use crate::state::VaultState;

pub enum ActionCalc {
  Deposit,
  Withdraw,
}

pub fn mul_u64_by_f32(a: u64, b: f32) -> u64 {
  let result = (a as f64) * (b as f64);
  result as u64
}

pub fn get_reward_ratio(vault_state_account: &VaultState) -> u64 {
  vault_state_account.total_deposits / vault_state_account.reward_factor
}
