use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount, Transfer, CloseAccount, transfer, close_account}, 
    associated_token::AssociatedToken
};

use crate::{state::{Details, NftRecord}, utils::calc_reward, StakeError};

#[derive(Accounts)]
pub struct Unstake<'info> {
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
        has_one = nft_mint,
        has_one = staker,
        close = staker
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
        close = staker
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
    pub fn transfer_token_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.stake_token_vault.to_account_info(),
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
    
        let cpi_program = self.token_program.clone().to_account_info();

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

    let Details {
        minimum_period,
        staking_ends_at,
        token_auth_bump,
        nft_auth_bump,
        ..
    } = **stake_details;

    let reward_record = &stake_details.reward;
    let reward_change_time_record = &stake_details.reward_change_time;
    let stake_details_key = stake_details.key();

    let staked_at = ctx.accounts.nft_record.staked_at;
    
    let (reward_tokens, current_time, is_eligible_for_reward) = calc_reward(
        staked_at, 
        minimum_period, 
        reward_record,
        reward_change_time_record,
        staking_ends_at
    ).unwrap();

    if is_eligible_for_reward {
        // Transfer Reward Tokens
        let token_auth_seed = &[&b"token-authority"[..], &stake_details_key.as_ref(), &[token_auth_bump]];
        transfer(
            ctx.accounts.transfer_token_ctx().with_signer(&[&token_auth_seed[..]]), 
            reward_tokens
        )?;
    }

    // Transfer NFT
    let nft_auth_seed = &[&b"nft-authority"[..], &stake_details_key.as_ref(), &[nft_auth_bump]];
    transfer(
        ctx.accounts.transfer_nft_ctx().with_signer(&[&nft_auth_seed[..]]), 
        1
    )?;

    // Close NFT Custody Account
    close_account(ctx.accounts.close_account_ctx().with_signer(&[&nft_auth_seed[..]]))?;
    
    let stake_details = &mut ctx.accounts.stake_details;

    // Delete stake weight and reduce staker count
    stake_details.update_staked_weight(staked_at, false)?; 
    stake_details.decrease_staker_count()?;

    // Decrease the balance in record
    stake_details.decrease_current_balance(staked_at, current_time)
}