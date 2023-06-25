use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount, MintTo, Transfer, CloseAccount, mint_to, transfer, close_account}, 
    associated_token::AssociatedToken
};

use crate::{state::{Details, NftRecord}, utils::calc_reward, StakeError};

#[derive(Accounts)]
pub struct Unstake<'info> {
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
        has_one = nft_mint,
        has_one = staker,
        close = staker
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
    pub reward_receive_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mint::decimals = 0,
        constraint = nft_mint.supply == 1 @ StakeError::TokenNotNFT,
    )]
    nft_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = staker,
        associated_token::mint = nft_mint,
        associated_token::authority = staker,
    )]
    nft_receive_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = nft_authority,
        constraint = nft_custody.amount == 1 @ StakeError::TokenAccountEmpty,
    )]
    pub nft_custody: Box<Account<'info, TokenAccount>>,

    /// CHECK: This account is not read or written
    #[account(
        seeds = [
            b"token-authority",
            stake_details.key().as_ref(),
        ],
        bump = stake_details.token_auth_bump
    )]
    pub token_authority: UncheckedAccount<'info>,

     /// CHECK: This account is not read or written
     #[account(
        seeds = [
            b"nft-authority",
            stake_details.key().as_ref()
        ],
        bump = stake_details.nft_auth_bump
    )]
    pub nft_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub staker: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>
}

impl<'info> Unstake<'info> {
    pub fn mint_token_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.reward_mint.to_account_info(),
            to: self.reward_receive_account.to_account_info(),
            authority: self.token_authority.to_account_info()
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }

    pub fn transfer_nft_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.nft_custody.to_account_info(),
            to: self.nft_receive_account.to_account_info(),
            authority: self.nft_authority.to_account_info()
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }

    pub fn close_account_ctx(&self)-> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.nft_custody.to_account_info(),
            destination: self.staker.to_account_info(),
            authority: self.nft_authority.to_account_info()
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

pub fn unstake_handler(ctx: Context<Unstake>) -> Result<()> {
    let stake_details = &ctx.accounts.stake_details;

    let staked_at = ctx.accounts.nft_record.staked_at;
    let minimum_stake_period = stake_details.minimum_period;
    let reward_emission = stake_details.reward;
    let staking_active = stake_details.is_active;
    let token_auth_bump = stake_details.token_auth_bump;
    let nft_auth_bump = stake_details.nft_auth_bump;
    let stake_details_key = stake_details.key();

    let (reward_tokens, _current_time, is_eligible_for_reward) = calc_reward(
        staked_at, 
        minimum_stake_period, 
        reward_emission,
    ).unwrap();

    let token_auth_seed = &[&b"token-authority"[..], &stake_details_key.as_ref(), &[token_auth_bump]];
    let nft_auth_seed = &[&b"nft-authority"[..], &stake_details_key.as_ref(), &[nft_auth_bump]];

    if is_eligible_for_reward && staking_active {
        // Mint Reward Tokens
        mint_to(
            ctx.accounts.mint_token_ctx().with_signer(&[&token_auth_seed[..]]), 
        reward_tokens
        )?;
    }

    // Transfer NFT
    transfer(
        ctx.accounts.transfer_nft_ctx().with_signer(&[&nft_auth_seed[..]]), 
        1
    )?;

    // Close NFT Custody Account
    close_account(ctx.accounts.close_account_ctx().with_signer(&[&nft_auth_seed[..]]))?;
    
    Ok(())
}