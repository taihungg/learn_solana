use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::{
    constant::{BANK_INFO_SEED, BANK_VAULT_SEED, USER_RESERVE_SEED},
    error::BankAppError,
    state::{BankInfo, UserReserve}, 
    transfer_helper::token_transfer_from_pda,
};

#[derive(Accounts)]
pub struct WithdrawToken<'info> {
    #[account(
        mut,
        seeds = [BANK_INFO_SEED],
        bump
    )]
    pub bank_info: Box<Account<'info, BankInfo>>,

    #[account(
        mut,
        seeds = [BANK_VAULT_SEED],
        bump,
    )]
    pub bank_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [USER_RESERVE_SEED, user.key().as_ref()],
        bump,
    )]
    pub user_reserve: Box<Account<'info, UserReserve>>,

    pub user: Signer<'info>,

    #[account(mut)]
    pub user_token_account: Box<Account<'info, TokenAccount>>,
    
    /// CHECK:
    pub token_program: Program<'info, Token>,
}

impl<'info> WithdrawToken<'info> {  
    pub fn process(ctx: Context<WithdrawToken>, withdraw_amount: u64) -> Result<()> {
        if ctx.accounts.bank_info.is_paused {
            return Err(BankAppError::BankAppPaused.into());
        }

        let user_reserve = &mut ctx.accounts.user_reserve;
        if user_reserve.deposited_amount < withdraw_amount {
            return Err(BankAppError::InsufficientFunds.into());
        }

        let pda_seeds: &[&[&[u8]]] = &[&[BANK_INFO_SEED, &[ctx.accounts.bank_info.bump]]];

        token_transfer_from_pda(
            ctx.accounts.bank_vault.to_account_info(),
            ctx.accounts.bank_info.to_account_info(),
            ctx.accounts.user_token_account.to_account_info(),
            &ctx.accounts.token_program,
            pda_seeds,
            withdraw_amount,
        )?;

        user_reserve.deposited_amount -= withdraw_amount;

        Ok(())
    }
}