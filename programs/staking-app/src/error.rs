use anchor_lang::prelude::*;

#[error_code]
pub enum StakingAppError {
    #[msg("Insufficient funds for withdrawal.")]
    InsufficientFunds
}