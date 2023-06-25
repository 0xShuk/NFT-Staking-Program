use anchor_lang::prelude::*;
use anchor_spl::token::{set_authority, SetAuthority, Token, Mint, spl_token::instruction::AuthorityType};

use crate::{state::Details, StakeError};

#[derive(Accounts)]
pub struct CloseStaking<'info> {
    #[account(
        mut,
        seeds = [
            b"stake", 
            stake_details.collection.as_ref(),
            stake_details.creator.as_ref()
        ],
        bump = stake_details.stake_bump,
        has_one = creator
    )]
    pub stake_details: Account<'info, Details>,

    #[account(
        mut,
        mint::authority = token_authority,
    )]
    pub token_mint: Account<'info, Mint>,

    /// CHECK: This account is not read or written
    #[account(
        seeds = [
            b"token-authority",
            stake_details.key().as_ref()
        ],
        bump = stake_details.token_auth_bump
    )]
    pub token_authority: UncheckedAccount<'info>,

    pub creator: Signer<'info>,
    pub token_program: Program<'info, Token>
}

impl<'info> CloseStaking<'info> {
    pub fn transfer_auth_ctx(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.token_mint.to_account_info(),
            current_authority: self.token_authority.to_account_info()
        };
    
        let cpi_program = self.token_program.to_account_info();

        CpiContext::new(cpi_program, cpi_accounts)
    }
}

pub fn close_staking_handler(ctx: Context<CloseStaking>) -> Result<()> {
    let stake_details = &ctx.accounts.stake_details;

    let staking_status = stake_details.is_active;
    let token_auth_bump = stake_details.token_auth_bump;
    let stake_details_key = stake_details.key();
    let creator = ctx.accounts.creator.key();

    require_eq!(staking_status, true, StakeError::StakingInactive);

    let token_auth_seed = &[&b"token-authority"[..], &stake_details_key.as_ref(), &[token_auth_bump]];

    set_authority(
        ctx.accounts.transfer_auth_ctx().with_signer(&[&token_auth_seed[..]]),
        AuthorityType::MintTokens,
        Some(creator)
    )?;

    ctx.accounts.stake_details.close_staking()
}