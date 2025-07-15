use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;

use crate::{
    constant::{SECOND_PER_YEAR, STAKING_APR},
    transfer_helper::{token_transfer_from_pda, token_transfer_from_user},
    error::StakingAppError,
};

#[derive(Accounts)]
pub struct StakeToken<'info> {
    // authority của vault, được xác thực bằng seeds và bump
    /// CHECK:
    #[account(
        seeds = [b"VAULT_AUTHORITY", mint.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    // kho chứa tiền của staking_app (tiền thực sự sẽ đổ về đây)
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = vault_authority,
    )]
    pub staking_vault: Box<Account<'info, TokenAccount>>,

    // tài khoản lưu thông tin stake của người dùng trong staking_app
    #[account(
        init_if_needed,
        seeds = [b"BANK_INFO_SEED", user.key().as_ref(), mint.key().as_ref()],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<UserTokenInfo>(),
    )]
    pub user_info: Box<Account<'info, UserTokenInfo>>,

    // người dùng stake token
    #[account(mut)]
    pub user: Signer<'info>,

    // người trả fee cho giao dịch
    #[account(mut)]
    pub payer: Signer<'info>,

    // nguồn tiền của người dùng
    #[account(mut)]
    pub user_ata: Box<Account<'info, TokenAccount>>,

    pub mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[account]
#[derive(Default)]
pub struct UserTokenInfo {
    pub amount: u64,
    pub last_update_time: u64,
}

impl<'info> StakeToken<'info> {
    pub fn process(ctx: Context<StakeToken>, amount: u64, is_stake: bool) -> Result<()> {
        // user_info này là thông tin stake của người dùng trong staking_app
        let user_info = &mut ctx.accounts.user_info;

        let current_time: u64 = Clock::get()?.unix_timestamp.try_into().unwrap();
        let pass_time = if user_info.last_update_time == 0 {
            0
        } else {
            current_time - user_info.last_update_time
        };

        user_info.amount += (user_info.amount * STAKING_APR * pass_time) / (100 * SECOND_PER_YEAR);
        user_info.last_update_time = current_time;

        if amount > 0 {
            if is_stake {
                token_transfer_from_user(
                    ctx.accounts.user_ata.to_account_info(),
                    &ctx.accounts.user,
                    ctx.accounts.staking_vault.to_account_info(),
                    &ctx.accounts.token_program,
                    amount,
                )?;
                user_info.amount += amount;
            } else {
                if user_info.amount < amount {
                    return err!(StakingAppError::InsufficientFunds);
                }
                
                let bump = ctx.bumps.vault_authority;
                let mint_key = ctx.accounts.mint.key();
                let authority_seeds: &[&[u8]] = &[b"VAULT_AUTHORITY", mint_key.as_ref(), &[bump]];
                let signer_seeds: &[&[&[u8]]] = &[&authority_seeds[..]];

                token_transfer_from_pda(
                    ctx.accounts.staking_vault.to_account_info(),
                    ctx.accounts.vault_authority.to_account_info(),
                    ctx.accounts.user_ata.to_account_info(),
                    &ctx.accounts.token_program,
                    signer_seeds,
                    amount,
                )?;
                
                user_info.amount -= amount;
            }
        }
        Ok(())
    }
}
