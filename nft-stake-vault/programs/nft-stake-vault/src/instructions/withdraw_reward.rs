use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount, Transfer, transfer}, 
    associated_token::AssociatedToken
};

use crate::{state::{Details, NftRecord}, utils::calc_reward, StakeError};

#[derive(Accounts)]
pub struct WithdrawReward<'info> {
    #[account(
        mut,
        seeds = [
            b"stake", 
            stake_details.collection.as_ref(),
            stake_details.creator.as_ref()
        ],
        bump = stake_details.stake_bump,
        has_one = reward_mint
    )]
    pub stake_details: Account<'info, Details>,

    #[account(
        mut,
        seeds = [
            b"nft-record", 
            stake_details.key().as_ref(),
            nft_record.nft_mint.as_ref(),
        ],
        bump = nft_record.bump,
        has_one = staker
    )]
    pub nft_record: Account<'info, NftRecord>,

    pub reward_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = reward_mint,
        associated_token::authority = token_authority
    )]
    pub stake_token_vault: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = staker,
        associated_token::mint = reward_mint,
        associated_token::authority = staker
    )]
    pub reward_receive_account: Account<'info, TokenAccount>,

    /// CHECK: This account is not read or written
    #[account(
        seeds = [
            b"token-authority",
            stake_details.key().as_ref(),
        ],
        bump = stake_details.token_auth_bump
    )]
    pub token_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub staker: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>
}

impl<'info> WithdrawReward<'info> {
    pub fn transfer_token_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.stake_token_vault.to_account_info(),
            to: self.reward_receive_account.to_account_info(),
            authority: self.token_authority.to_account_info()
        };
    
        let cpi_program = self.token_program.to_account_info();

        CpiContext::new(cpi_program, cpi_accounts)
    }
}

pub fn withdraw_reward_handler(ctx: Context<WithdrawReward>) -> Result<()> {
    let stake_details = &ctx.accounts.stake_details;

    let Details {
        minimum_period,
        staking_ends_at,
        is_active: staking_status,
        token_auth_bump,
        ..
    } = **stake_details;

    let reward_record = &stake_details.reward;
    let reward_change_time_record = &stake_details.reward_change_time;
    let stake_details_key = stake_details.key();

    let staked_at = ctx.accounts.nft_record.staked_at;
    
    require_eq!(staking_status, true, StakeError::StakingInactive);
    require_gte!(staking_ends_at, staked_at, StakeError::StakingIsOver);

    let (reward_tokens, current_time, is_eligible_for_reward) = calc_reward(
        staked_at, 
        minimum_period, 
        reward_record,
        reward_change_time_record,
        staking_ends_at
    ).unwrap();

    if is_eligible_for_reward {
        let authority_seed = &[&b"token-authority"[..], &stake_details_key.as_ref(), &[token_auth_bump]];

        transfer(
            ctx.accounts.transfer_token_ctx().with_signer(&[&authority_seed[..]]), 
            reward_tokens)?;
    } else {
        return err!(StakeError::IneligibleForReward);
    }

    ctx.accounts.nft_record.staked_at = current_time;

    let stake_details = &mut ctx.accounts.stake_details;

    // Remove previous stake weight
    stake_details.update_staked_weight(staked_at, false)?;

    // Add new stake weight
    stake_details.update_staked_weight(current_time, true)?;

    // Decrease the balance in record
    stake_details.decrease_current_balance(staked_at, current_time)
 
}