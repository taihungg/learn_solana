use anchor_lang::prelude::*;

pub mod constant;
pub mod error;
pub mod instructions;
pub mod transfer_helper;

pub use instructions::*;

declare_id!("7SjKko5YDfUNwyjPd1eQSG8spdzbByhv1wVqgbMwbPPA");

#[program]
pub mod staking_app {
    use super::*;

    pub fn stake_sol(ctx: Context<StakeSol>, amount: u64, is_stake: bool) -> Result<()> {
        return StakeSol::process(ctx, amount, is_stake);
    }

    pub fn stake_token(ctx: Context<StakeToken>, amount: u64, is_stake: bool) -> Result<()> {
        return StakeToken::process(ctx, amount, is_stake);
    }
}