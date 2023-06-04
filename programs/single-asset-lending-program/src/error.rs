use anchor_lang::prelude::*;

#[error_code]
pub enum LendingError {
  #[msg("Interest rate must be between 0 and 1")]
  InterestRateOutOfBounds,

  #[msg("Borrow percentage must be between 0 and 1")]
  BorrowMaxOutOfBounds,

  #[msg("Attempt to borrow over maximum")]
  CannotBorrowOverMax,
}