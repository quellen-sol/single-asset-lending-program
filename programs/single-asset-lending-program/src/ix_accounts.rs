use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

pub const VAULT_SEED: &str = "vault";
pub const VAULT_STATE_SEED: &str = "state";
pub const VAULT_REWARDS_SEED: &str = "rewards";
pub const USER_VAULT_STATE_SEED: &str = "userVault";

#[derive(Accounts)]
pub struct CreateVault<'info> {
  #[account(mut)]
  pub payer: Signer<'info>,

  pub vault_mint: Account<'info, Mint>,

  #[account(init, payer = payer, token::mint = vault_mint, token::authority = payer, seeds = [
    VAULT_SEED.as_bytes(),
    vault_mint.key().as_ref(),
  ], bump)]
  pub vault_account: Account<'info, TokenAccount>,

  #[account(init, payer = payer, token::mint = vault_mint, token::authority = payer, seeds = [
    VAULT_REWARDS_SEED.as_bytes(),
    vault_account.key().as_ref(),
  ], bump)]
  pub vault_rewards_account: Account<'info, TokenAccount>,

  #[account(init, payer = payer, space = VaultState::SIZE, seeds = [
    VAULT_STATE_SEED.as_bytes(),
    vault_account.key().as_ref(),
  ], bump)]
  pub vault_state_account: Box<Account<'info, VaultState>>,

  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, Token>,
  pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DepositToVault<'info> {
  #[account(mut)]
  pub payer: Signer<'info>,

  pub vault_mint: Account<'info, Mint>,

  #[account(mut, token::mint = vault_mint, seeds = [
    VAULT_SEED.as_bytes(),
    vault_mint.key().as_ref(),
  ], bump)]
  pub vault_account: Account<'info, TokenAccount>,

  #[account(mut, seeds = [
    VAULT_STATE_SEED.as_bytes(),
    vault_account.key().as_ref(),
  ], bump)]
  pub vault_state_account: Box<Account<'info, VaultState>>,

  #[account(init_if_needed, payer = payer, space = UserState::SIZE, seeds = [
    USER_VAULT_STATE_SEED.as_bytes(),
    vault_account.key().as_ref(),
  ], bump)]
  pub user_state_account: Box<Account<'info, UserState>>,

  #[account(mut, token::mint = vault_mint)]
  pub user_token_account: Account<'info, TokenAccount>,

  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Borrow<'info> {
  pub vault_mint: Account<'info, Mint>,

  #[account(mut, token::mint = vault_mint, seeds = [
    VAULT_SEED.as_bytes(),
    vault_mint.key().as_ref(),
  ], bump)]
  pub vault_account: Account<'info, TokenAccount>,

  #[account(mut, seeds = [
    VAULT_STATE_SEED.as_bytes(),
    vault_account.key().as_ref(),
  ], bump)]
  pub vault_state_account: Account<'info, VaultState>,

  #[account(mut, seeds = [
    USER_VAULT_STATE_SEED.as_bytes(),
    vault_account.key().as_ref(),
  ], bump)]
  pub user_state_account: Account<'info, UserState>,

  #[account(mut, token::mint = vault_mint)]
  pub user_token_acccount: Account<'info, TokenAccount>,

  pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Repay<'info> {
  #[account(mut)]
  pub payer: Signer<'info>,

  pub vault_mint: Account<'info, Mint>,

  #[account(mut, token::mint = vault_mint, seeds = [
    VAULT_SEED.as_bytes(),
    vault_mint.key().as_ref(),
  ], bump)]
  pub vault_account: Account<'info, TokenAccount>,

  #[account(mut, seeds = [
    USER_VAULT_STATE_SEED.as_bytes(),
    vault_account.key().as_ref(),
  ], bump)]
  pub user_state_account: Box<Account<'info, UserState>>,

  #[account(mut, seeds = [
    VAULT_STATE_SEED.as_bytes(),
    vault_account.key().as_ref(),
  ], bump)]
  pub vault_state_account: Box<Account<'info, VaultState>>,

  #[account(mut, token::mint = vault_mint, seeds = [
    VAULT_REWARDS_SEED.as_bytes(),
    vault_account.key().as_ref(),
  ], bump)]
  pub vault_rewards_account: Account<'info, TokenAccount>,

  #[account(mut, token::mint = vault_mint)]
  pub user_token_acccount: Account<'info, TokenAccount>,

  pub token_program: Program<'info, Token>,
}
