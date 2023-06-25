use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount, Transfer, transfer}, 
    associated_token::AssociatedToken
};

use crate::{state::Details, StakeError, utils::calc_total_emission};

#[derive(Accounts)]
pub struct InitStaking<'info> {
    #[account(
        init, 
        payer = creator, 
        space = Details::LEN,
        seeds = [
            b"stake", 
            collection_address.key().as_ref(),
            creator.key().as_ref()
        ],
        bump
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
        init_if_needed,
        payer = creator,
        associated_token::mint = token_mint,
        associated_token::authority = token_authority,
    )]
    pub stake_token_vault: Account<'info, TokenAccount>,

    #[account(
        mint::decimals = 0,
    )]
    pub collection_address: Account<'info, Mint>,

    #[account(mut)]
    pub creator: Signer<'info>,

    /// CHECK: This account is not read or written
    #[account(
        seeds = [
            b"token-authority",
            stake_details.key().as_ref()
        ],
        bump
    )]
    pub token_authority: UncheckedAccount<'info>,

    /// CHECK: This account is not read or written
    #[account(
        seeds = [
            b"nft-authority",
            stake_details.key().as_ref()
        ],
        bump
    )]
    pub nft_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>
}

impl<'info> InitStaking<'info> {
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

pub fn init_staking_handler(
    ctx: Context<InitStaking>, 
    reward: u64, 
    minimum_period: i64,
    staking_starts_at: i64,
    staking_ends_at: i64,
    max_stakers_count: u64
) -> Result<()> {
    let clock = Clock::get().unwrap();
    let current_time = clock.unix_timestamp;

    require_gte!(minimum_period, 0, StakeError::NegativePeriodValue);
    require_gt!(staking_ends_at, current_time, StakeError::InvalidStakeEndTime);
    require_gt!(staking_ends_at, staking_starts_at, StakeError::InvalidStakeEndTime);

    let reward_mint = ctx.accounts.token_mint.key();
    let collection = ctx.accounts.collection_address.key();
    let creator = ctx.accounts.creator.key();
    let stake_bump = *ctx.bumps.get("stake_details").ok_or(StakeError::StakeBumpError)?;
    let token_auth_bump = *ctx.bumps.get("token_authority").ok_or(StakeError::StakeBumpError)?;
    let nft_auth_bump = *ctx.bumps.get("nft_authority").ok_or(StakeError::StakeBumpError)?;

    let total_emission = calc_total_emission(reward, max_stakers_count, staking_starts_at, staking_ends_at)?;

    transfer(ctx.accounts.transfer_token_ctx(), total_emission)?;

    let stake_details = &mut ctx.accounts.stake_details;

    **stake_details = Details::init(
        creator,
        reward_mint, 
        collection,
        reward,
        max_stakers_count,
        staking_starts_at,
        staking_ends_at,
        minimum_period,
        stake_bump,
        token_auth_bump,
        nft_auth_bump,
        total_emission
    );


    Ok(())
}