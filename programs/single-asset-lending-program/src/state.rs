// Accounts:
// Vault
// Vault State
// Vault Rewards

use anchor_lang::prelude::*;

#[account]
pub struct VaultState {
  pub total_deposits: u64,
  pub interest_rate: f32,
  pub borrow_percentage_per_user: f32,
  pub reward_factor: u64,
}

impl VaultState {
  pub const SIZE: usize = 8 + 8 + 4 + 4;
}

#[account]
pub struct UserState {
  pub total_deposits: u64,
  pub total_borrows: u64,
  pub amount_to_repay: u64,
}

impl UserState {
  pub const SIZE: usize = 8 + 8 + 8 + 8;
}