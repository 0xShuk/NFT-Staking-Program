use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, SetAuthority, set_authority, spl_token::instruction::AuthorityType}, 
};

use crate::{state::Details, StakeError};

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

    #[account(
        mut,
        mint::authority = creator
    )]
    pub token_mint: Account<'info, Mint>,

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
    pub system_program: Program<'info, System>
}

impl<'info> InitStaking<'info> {
    pub fn transfer_auth_ctx(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.token_mint.to_account_info(),
            current_authority: self.creator.to_account_info()
        };
    
        let cpi_program = self.token_program.to_account_info();

        CpiContext::new(cpi_program, cpi_accounts)
    }
}

pub fn init_staking_handler(ctx: Context<InitStaking>, reward: u64, minimum_period: i64) -> Result<()> {
   
    require_gte!(minimum_period, 0, StakeError::NegativePeriodValue);

    let reward_mint = ctx.accounts.token_mint.key();
    let collection = ctx.accounts.collection_address.key();
    let creator = ctx.accounts.creator.key();
    let stake_bump = *ctx.bumps.get("stake_details").ok_or(StakeError::StakeBumpError)?;
    let token_auth_bump = *ctx.bumps.get("token_authority").ok_or(StakeError::TokenAuthBumpError)?;
    let nft_auth_bump = *ctx.bumps.get("nft_authority").ok_or(StakeError::NftAuthBumpError)?;
    let token_authority = ctx.accounts.token_authority.key();

    set_authority(
        ctx.accounts.transfer_auth_ctx(),
        AuthorityType::MintTokens,
        Some(token_authority)
    )?;

    let stake_details = &mut ctx.accounts.stake_details;

    **stake_details = Details::init(
        creator,
        reward_mint, 
        reward, 
        collection,
        minimum_period,
        stake_bump,
        token_auth_bump,
        nft_auth_bump
    );

    Ok(())
}