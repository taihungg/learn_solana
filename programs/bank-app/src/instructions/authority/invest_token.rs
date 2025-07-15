use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use staking_app::{cpi, program::StakingApp};

use crate::{
    constant::{VAULT_AUTHORITY},
    error::BankAppError,
    state::BankInfo,
};

#[derive(Accounts)]
pub struct InvestToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(has_one = authority)]
    pub bank_info: Account<'info, BankInfo>,

    #[account(mut)]
    pub bank_vault: Account<'info, TokenAccount>,

    /// CHECK:
    #[account(
        seeds = [VAULT_AUTHORITY, mint.key().as_ref()],
        bump
    )]
    pub bank_vault_authority: UncheckedAccount<'info>,

    pub staking_program: Program<'info, StakingApp>,

    /// CHECK:
    pub staking_vault_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub staking_vault: Account<'info, TokenAccount>,

    /// CHECK:
    #[account(mut)]
    pub user_info: AccountInfo<'info>,

    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> InvestToken<'info> {
    pub fn process(ctx: Context<InvestToken>, amount: u64, is_stake: bool) -> Result<()> {
        if ctx.accounts.bank_info.is_paused {
            return Err(BankAppError::BankAppPaused.into());
        }
        if ctx.accounts.bank_vault.amount < amount {
            return Err(BankAppError::InsufficientFunds.into());
        }

        let bump = ctx.bumps.bank_vault_authority;
        let mint_key = ctx.accounts.mint.key();
        let authority_seeds: &[&[u8]] = &[VAULT_AUTHORITY, mint_key.as_ref(), &[bump]];
        let signer_seeds: &[&[&[u8]]] = &[&authority_seeds[..]];

        cpi::stake_token(
            CpiContext::new_with_signer(
                ctx.accounts.staking_program.to_account_info(),
                cpi::accounts::StakeToken {
                    vault_authority: ctx.accounts.staking_vault_authority.to_account_info(),
                    staking_vault: ctx.accounts.staking_vault.to_account_info(),
                    user_info: ctx.accounts.user_info.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),

                    user: ctx.accounts.bank_vault_authority.to_account_info(),
                    payer: ctx.accounts.authority.to_account_info(),
                    user_ata: ctx.accounts.bank_vault.to_account_info(),

                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
            is_stake,
        )?;

        msg!(
            "Bank app successfully invested {} tokens into the staking protocol.",
            amount
        );
        Ok(())
    }
}