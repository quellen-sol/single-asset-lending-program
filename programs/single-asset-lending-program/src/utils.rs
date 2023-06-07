use crate::state::VaultState;

pub fn mul_u64_by_f32(a: u64, b: f32) -> u64 {
  let result = (a as f64) * (b as f64);
  result as u64
}

pub fn div_u64_by_f64(numerator: u64, denominator: f64) -> f64 {
  (numerator as f64) / (denominator as f64)
}

pub fn get_reward_ratio(vault_state_account: &VaultState) -> f64 {
  div_u64_by_f64(vault_state_account.total_deposits, vault_state_account.reward_factor)
}
