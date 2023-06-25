use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Transfer, Token, TokenAccount, Mint};

use crate::{state::Details, StakeError, utils::calc_actual_balance};

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

    pub token_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = creator
    )]
    pub token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = token_authority,
    )]
    pub stake_token_vault: Account<'info, TokenAccount>,

    /// CHECK: This account is not read or written
    #[account(
        seeds = [
            b"token-authority",
            stake_details.key().as_ref()
        ],
        bump
    )]
    pub token_authority: UncheckedAccount<'info>,

    pub creator: Signer<'info>,
    pub token_program: Program<'info, Token>
}

impl<'info> CloseStaking<'info> {
    pub fn transfer_token_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.stake_token_vault.to_account_info(),
            to: self.token_account.to_account_info(),
            authority: self.token_authority.to_account_info()
        };
    
        let cpi_program = self.token_program.to_account_info();

        CpiContext::new(cpi_program, cpi_accounts)
    }
}

pub fn close_staking_handler(ctx: Context<CloseStaking>) -> Result<()> {
    let stake_details = &ctx.accounts.stake_details;
    let current_time = Clock::get().unwrap().unix_timestamp;

    let Details {
        current_stakers_count,
        staking_ends_at,
        staked_weight,
        is_active: staking_status,
        token_auth_bump,
        ..
    } = **stake_details;

    let current_reward = *stake_details.reward.last().unwrap();
    let last_reward_change_time = *stake_details.reward_change_time.last().unwrap();
    let stake_details_key = stake_details.key();

    let current_balance = ctx.accounts.stake_token_vault.amount;
    
    require_eq!(staking_status, true, StakeError::StakingInactive);

    let (current_actual_balance, _new_staked_weight) = calc_actual_balance(
        current_stakers_count,
        staked_weight,
        current_reward,
        last_reward_change_time,
        staking_ends_at,
        current_time,
        current_balance,
        None
    )?;

    // Transfer remaining balance back to the creator
    let token_auth_seed = &[&b"token-authority"[..], &stake_details_key.as_ref(), &[token_auth_bump]];
    transfer(
        ctx.accounts.transfer_token_ctx().with_signer(&[&token_auth_seed[..]]), 
        current_actual_balance
    )?;

    let stake_details = &mut ctx.accounts.stake_details;

    stake_details.close_staking();

    // Allow stakers to instantly withdraw their NFTs
    stake_details.minimum_period = 0;

    // If the staking end time is more than the current time then change it to current
    // This is done to avoid accrual of any new stake rewards
    stake_details.staking_ends_at = if staking_ends_at > current_time {
        current_time
    } else {
        staking_ends_at
    };

    Ok(())
}