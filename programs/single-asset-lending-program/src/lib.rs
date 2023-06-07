use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Transfer};
use error::LendingError;
use ix_accounts::*;
use utils::*;

mod error;
mod ix_accounts;
mod state;
mod utils;

declare_id!("F5dLpWLFYuEGEZZNf2RQt8sJnr5mzgpDhY3npuz8JggS");

#[program]
pub mod single_asset_lending_program {
  use super::*;

  pub fn create_vault(
    ctx: Context<CreateVault>,
    interest_rate: f32,
    max_borrow_percentage: f32,
  ) -> Result<()> {
    require!(
      max_borrow_percentage <= 1.0,
      LendingError::BorrowMaxOutOfBounds
    );
    require!(
      max_borrow_percentage >= 0.0,
      LendingError::BorrowMaxOutOfBounds
    );
    require!(interest_rate <= 1.0, LendingError::InterestRateOutOfBounds);
    require!(interest_rate >= 0.0, LendingError::InterestRateOutOfBounds);

    let vault_state = &mut ctx.accounts.vault_state_account;

    vault_state.total_deposits = 0;
    vault_state.borrow_percentage_per_user = max_borrow_percentage;
    vault_state.interest_rate = interest_rate;
    vault_state.reward_factor = 1.0; // Avoid div by 0 in get_reward_ratio()
    Ok(())
  }

  pub fn deposit(ctx: Context<DepositToVault>, amount: u64) -> Result<()> {
    let user_state = &mut ctx.accounts.user_state_account;
    let vault_state = &mut ctx.accounts.vault_state_account;

    let transfer_cpi = CpiContext::new(
      ctx.accounts.token_program.to_account_info(),
      Transfer {
        authority: ctx.accounts.payer.to_account_info(),
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.vault_account.to_account_info(),
      },
    );

    transfer(transfer_cpi, amount)?;

    user_state.total_deposits += amount;

    let mut reward_ratio = get_reward_ratio(&vault_state);
    if reward_ratio == 0.0 {
      reward_ratio = 0.5;
    }

    vault_state.total_deposits += amount;
    vault_state.reward_factor += reward_ratio * (amount as f64);
    Ok(())
  }

  pub fn borrow(ctx: Context<Borrow>, bump: u8, amount: u64) -> Result<()> {
    let total_borrow_after = ctx.accounts.user_state_account.total_borrows + amount;
    let max_user_borrows = (ctx.accounts.user_state_account.total_deposits as f64)
      * (ctx.accounts.vault_state_account.borrow_percentage_per_user as f64);
    require!(
      total_borrow_after <= max_user_borrows as u64,
      LendingError::CannotBorrowOverMax
    );

    let user_state = &mut ctx.accounts.user_state_account;

    let mint_key = ctx.accounts.vault_mint.key();
    let seeds = [VAULT_SEED.as_bytes(), mint_key.as_ref(), &[bump]];
    let signer: &[&[&[u8]]] = &[&seeds[..]];

    let transfer_cpi = CpiContext::new_with_signer(
      ctx.accounts.token_program.to_account_info(),
      Transfer {
        authority: ctx.accounts.vault_account.to_account_info(),
        from: ctx.accounts.vault_account.to_account_info(),
        to: ctx.accounts.user_token_acccount.to_account_info(),
      },
      signer,
    );

    transfer(transfer_cpi, amount)?;

    user_state.total_borrows += amount;
    user_state.amount_to_repay +=
      amount + mul_u64_by_f32(amount, ctx.accounts.vault_state_account.interest_rate);

    Ok(())
  }

  pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
    let user_state = &mut ctx.accounts.user_state_account;

    /*
      B = total_borrows
      R = total_repays
      R = B * (1 + I)
      R = B + BI

      BI = Total Interest, which needs to go to rewards = R - B

      Need to calculate what part of `amount` (A) goes to rewards

      % Of total Repay = A / R

      % of amount to go to rewards = P * BI
      = (A / R) * (R - B)
      = A - BA/R
      = A * (1 - B/R)
    */
    let part_rewards_ratio =
      1.0 - (user_state.total_borrows as f64) / (user_state.amount_to_repay as f64);
    let part_for_rewards = ((amount as f64) * part_rewards_ratio) as u64;
    let part_for_vault = amount - part_for_rewards;

    let vault_transfer_cpi = CpiContext::new(
      ctx.accounts.token_program.to_account_info(),
      Transfer {
        authority: ctx.accounts.payer.to_account_info(),
        from: ctx.accounts.user_token_acccount.to_account_info(),
        to: ctx.accounts.vault_account.to_account_info(),
      },
    );

    transfer(vault_transfer_cpi, part_for_vault)?;

    let rewards_transfer_cpi = CpiContext::new(
      ctx.accounts.token_program.to_account_info(),
      Transfer {
        authority: ctx.accounts.payer.to_account_info(),
        from: ctx.accounts.user_token_acccount.to_account_info(),
        to: ctx.accounts.vault_rewards_account.to_account_info(),
      },
    );

    transfer(rewards_transfer_cpi, part_for_rewards)?;

    user_state.amount_to_repay -= amount;
    user_state.total_borrows -= amount;
    Ok(())
  }

  pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state_account;
    let user_state = &mut ctx.accounts.user_state_account;

    require!(
      user_state.total_borrows == 0,
      LendingError::WithdrawWithBorrows
    );

    let mint_key = ctx.accounts.vault_mint.key();
    let seeds = &[VAULT_SEED.as_bytes(), mint_key.as_ref()];
    let signer = [&seeds[..]];

    let vault_to_user_cpi = CpiContext::new_with_signer(
      ctx.accounts.token_program.to_account_info(),
      Transfer {
        authority: ctx.accounts.vault_account.to_account_info(),
        from: ctx.accounts.vault_account.to_account_info(),
        to: ctx.accounts.user_token_acccount.to_account_info(),
      },
      &signer,
    );

    transfer(vault_to_user_cpi, amount)?;

    // Calculate rewards to include with withdraw
    let reward_ratio = get_reward_ratio(vault_state);

    let amount_rewards = div_u64_by_f64(amount, reward_ratio) as u64;

    let vault_key = ctx.accounts.vault_account.key();
    let seeds = &[VAULT_REWARDS_SEED.as_bytes(), vault_key.as_ref()];
    let signer = [&seeds[..]];

    let rewards_to_user_cpi = CpiContext::new_with_signer(
      ctx.accounts.token_program.to_account_info(),
      Transfer {
        authority: ctx.accounts.vault_rewards_account.to_account_info(),
        from: ctx.accounts.vault_rewards_account.to_account_info(),
        to: ctx.accounts.user_token_acccount.to_account_info(),
      },
      &signer,
    );

    transfer(rewards_to_user_cpi, amount_rewards)?;

    vault_state.total_deposits -= amount;
    user_state.total_deposits -= amount;

    Ok(())
  }
}
