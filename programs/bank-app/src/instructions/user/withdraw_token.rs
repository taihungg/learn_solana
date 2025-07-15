use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};

use crate::{
    constant::{BANK_INFO_SEED, BANK_VAULT_SEED, USER_RESERVE_SEED},
    error::BankAppError,
    state::{BankInfo, UserReserve}, 
    transfer_helper::token_transfer_from_pda,
};

#[derive(Accounts)]
pub struct WithdrawToken<'info> {
    #[account(
        seeds = [BANK_INFO_SEED], 
        bump
    )]
    pub bank_info: Box<Account<'info, BankInfo>>,

    /// CHECK:
    #[account(
        seeds = [BANK_VAULT_SEED],
        bump,
    )]
    pub bank_vault: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = bank_vault,
    )]
    pub bank_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [USER_RESERVE_SEED, user.key().as_ref(), mint.key().as_ref()],
        bump,
    )]
    pub user_reserve: Box<Account<'info, UserReserve>>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub user_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub mint: Account<'info, Mint>,
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

        let bump = ctx.bumps.bank_vault;
        let vault_seeds: &[&[u8]] = &[BANK_VAULT_SEED, &[bump]];
        let signer_seeds: &[&[&[u8]]] = &[&vault_seeds[..]];

        token_transfer_from_pda(
            ctx.accounts.bank_ata.to_account_info(),
            ctx.accounts.bank_vault.to_account_info(),
            ctx.accounts.user_ata.to_account_info(),
            &ctx.accounts.token_program,
            signer_seeds,
            withdraw_amount,
        )?;

        user_reserve.deposited_amount -= withdraw_amount;

        Ok(())
    }
}