use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Transfer};
use error::LendingError;
use ix_accounts::*;
use utils::*;

mod error;
mod ix_accounts;
mod state;
mod utils;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

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
    Ok(())
  }

  pub fn deposit_to_vault(ctx: Context<DepositToVault>, amount: u64) -> Result<()> {
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
    vault_state.total_deposits += amount;
    Ok(())
  }

  pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
    let total_borrow_after = ctx.accounts.user_state_account.total_borrows + amount;
    let max_user_borrows = (ctx.accounts.user_state_account.total_deposits as f64)
      * (ctx.accounts.vault_state_account.borrow_percentage_per_user as f64);
    require!(
      total_borrow_after <= max_user_borrows as u64,
      LendingError::CannotBorrowOverMax
    );

    let user_state = &mut ctx.accounts.user_state_account;
    let vault_account_key = ctx.accounts.vault_account.key();

    let seeds = &[vault_account_key.as_ref()];
    let signer = [&seeds[..]];
    let transfer_cpi = CpiContext::new_with_signer(
      ctx.accounts.token_program.to_account_info(),
      Transfer {
        authority: ctx.accounts.vault_account.to_account_info(),
        from: ctx.accounts.vault_account.to_account_info(),
        to: ctx.accounts.user_token_acccount.to_account_info(),
      },
      &signer,
    );

    transfer(transfer_cpi, amount)?;

    user_state.total_borrows += amount;
    user_state.amount_to_repay +=
      amount + calculate_interest(amount, ctx.accounts.vault_state_account.interest_rate);
    Ok(())
  }

  // pub fn repay(_ctx: Context<>) -> Result<()> {
  //   Ok(())
  // }
}
