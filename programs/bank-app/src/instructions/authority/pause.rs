use anchor_lang::prelude::*;

use crate::{
    constant::BANK_INFO_SEED,
    state::BankInfo,
};

#[derive(Accounts)]
pub struct Pause<'info> {
    #[account(
        mut,
        seeds = [BANK_INFO_SEED],
        bump,
        constraint = bank_info.authority == authority.key()
    )]
    pub bank_info: Box<Account<'info, BankInfo>>,

    pub authority: Signer<'info>,
}

impl<'info> Pause<'info> {
    pub fn process(ctx: Context<Pause>, pause: bool) -> Result<()> {
        ctx.accounts.bank_info.is_paused = pause;
        if pause {
            msg!("Bank app paused by authority: {}", ctx.accounts.authority.key());
        } else {
            msg!("Bank app resumed by authority: {}", ctx.accounts.authority.key());
        }
        Ok(())
    }
}
