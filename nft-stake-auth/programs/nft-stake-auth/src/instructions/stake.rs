use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, transfer, TokenAccount, Transfer}, 
    metadata::{MasterEditionAccount, MetadataAccount, Metadata}, 
    associated_token::AssociatedToken
};

use crate::{state::{Details, NftRecord}, StakeError};

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(
        seeds = [
            b"stake", 
            stake_details.collection.as_ref(),
            stake_details.creator.as_ref()
        ],
        bump = stake_details.stake_bump
    )]
    pub stake_details: Account<'info, Details>,

    #[account(
        init,
        payer = signer,
        space = NftRecord::LEN,
        seeds = [
            b"nft-record", 
            stake_details.key().as_ref(),
            nft_mint.key().as_ref(),
        ],
        bump
    )]
    pub nft_record: Account<'info, NftRecord>,

    #[account(
        mint::decimals = 0,
        constraint = nft_mint.supply == 1 @ StakeError::TokenNotNFT
    )]
    nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = signer,
        constraint = nft_token.amount == 1 @ StakeError::TokenAccountEmpty
    )]
    nft_token: Account<'info, TokenAccount>,

    #[account(
        seeds = [
            b"metadata",
            Metadata::id().as_ref(),
            nft_mint.key().as_ref()
        ],
        seeds::program = Metadata::id(),
        bump,
        constraint = nft_metadata.collection.as_ref().unwrap().verified @ StakeError::CollectionNotVerified,
        constraint = nft_metadata.collection.as_ref().unwrap().key == stake_details.collection @ StakeError::InvalidCollection
    )]
    nft_metadata: Box<Account<'info, MetadataAccount>>,

    #[account(
        seeds = [
            b"metadata",
            Metadata::id().as_ref(),
            nft_mint.key().as_ref(),
            b"edition"
        ],
        seeds::program = Metadata::id(),
        bump
    )]
    nft_edition: Box<Account<'info, MasterEditionAccount>>,

    /// CHECK: This account is not read or written
    #[account(
        seeds = [
            b"nft-authority",
            stake_details.key().as_ref()
        ],
        bump = stake_details.nft_auth_bump
    )]
    pub nft_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = nft_mint,
        associated_token::authority = nft_authority
    )]
    pub nft_custody: Account<'info, TokenAccount>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>
}

impl<'info> Stake<'info> {
    pub fn transfer_nft_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.nft_token.to_account_info(),
            to: self.nft_custody.to_account_info(),
            authority: self.signer.to_account_info()
        };
    
        let cpi_program = self.token_program.to_account_info();

        CpiContext::new(cpi_program, cpi_accounts)
    }
}

pub fn stake_handler(ctx: Context<Stake>) -> Result<()> {

    let staking_status = ctx.accounts.stake_details.is_active;
    
    require_eq!(staking_status, true, StakeError::StakingInactive);

    let staker = ctx.accounts.signer.key();
    let nft_mint = ctx.accounts.nft_mint.key();
    let bump = *ctx.bumps.get("nft_record").ok_or(StakeError::NftBumpError)?;

    transfer(ctx.accounts.transfer_nft_ctx(), 1)?;

    let nft_record = &mut ctx.accounts.nft_record;
    **nft_record = NftRecord::init(staker, nft_mint, bump);

    Ok(())
}