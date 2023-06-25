use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Transfer, Token, TokenAccount, Mint};

use crate::{state::Details, StakeError};

#[derive(Accounts)]
pub struct AddFunds<'info> {
    #[account(
        mut,
        seeds = [
            b"stake", 
            stake_details.collection.as_ref(),
            stake_details.creator.as_ref()
        ],
        bump = stake_details.stake_bump,
        has_one = creator,
        has_one = reward_mint
    )]
    pub stake_details: Account<'info, Details>,

    pub reward_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = reward_mint,
        associated_token::authority = creator
    )]
    pub token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = reward_mint,
        associated_token::authority = token_authority,
    )]
    pub stake_token_vault: Account<'info, TokenAccount>,

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

impl<'info> AddFunds<'info> {
    pub fn transfer_token_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.token_account.to_account_info(),
            to: self.stake_token_vault.to_account_info(),
            authority: self.creator.to_account_info()
        };
    
        let cpi_program = self.token_program.to_account_info();

        CpiContext::new(cpi_program, cpi_accounts)
    }
}

pub fn add_funds_handler(ctx: Context<AddFunds>, amount: u64) -> Result<()> {
    let stake_status = ctx.accounts.stake_details.is_active;

    require_eq!(stake_status, true, StakeError::StakingInactive);

    transfer(ctx.accounts.transfer_token_ctx(), amount)?;
    ctx.accounts.stake_details.increase_current_balance(amount)
}