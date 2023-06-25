use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount, MintTo, mint_to}, 
    associated_token::AssociatedToken
};

use crate::{state::{Details, NftRecord}, utils::calc_reward, StakeError};

#[derive(Accounts)]
pub struct WithdrawReward<'info> {
    #[account(
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

    #[account(
        mut,
        mint::authority = token_authority,
    )]
    pub reward_mint: Account<'info, Mint>,

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
        bump
    )]
    pub token_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub staker: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>
}

impl<'info> WithdrawReward<'info> {
    pub fn mint_token_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.reward_mint.to_account_info(),
            to: self.reward_receive_account.to_account_info(),
            authority: self.token_authority.to_account_info()
        };
    
        let cpi_program = self.token_program.to_account_info();

        CpiContext::new(cpi_program, cpi_accounts)
    }
}

pub fn withdraw_reward_handler(ctx: Context<WithdrawReward>) -> Result<()> {
    let stake_details = &ctx.accounts.stake_details;

    let staked_at = ctx.accounts.nft_record.staked_at;
    let minimum_stake_period = stake_details.minimum_period;
    let reward_emission = stake_details.reward;
    let staking_status = stake_details.is_active;
    let token_auth_bump = stake_details.token_auth_bump;
    let stake_details_key = stake_details.key();

    require_eq!(staking_status, true, StakeError::StakingInactive);

    let (reward_tokens, current_time, is_eligible_for_reward) = calc_reward(
        staked_at, 
        minimum_stake_period, 
        reward_emission,
    ).unwrap();

    let authority_seed = &[&b"token-authority"[..], &stake_details_key.as_ref(), &[token_auth_bump]];
 
    if is_eligible_for_reward {
        mint_to(
            ctx.accounts.mint_token_ctx().with_signer(&[&authority_seed[..]]),
             reward_tokens
        )?;
    } else {
        return err!(StakeError::IneligibleForReward);
    }

    ctx.accounts.nft_record.staked_at = current_time;
    
    Ok(())
}