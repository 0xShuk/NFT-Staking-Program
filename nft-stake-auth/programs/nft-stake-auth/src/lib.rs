use anchor_lang::prelude::*;

mod instructions;
mod state;
mod utils;

use instructions::*;

declare_id!("8AJVDu2KYFQZuW5AK8d9VXbEkowvDu22AUCordG4ZPre");

#[program]
pub mod nft_stake_auth {
    use super::*;

    pub fn init_staking(
        ctx: Context<InitStaking>, 
        reward: u64, 
        minimum_period: i64
    ) -> Result<()> {
        init_staking_handler(ctx, reward, minimum_period)
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        stake_handler(ctx)
    }

    pub fn withdraw_reward(ctx: Context<WithdrawReward>) -> Result<()> {
        withdraw_reward_handler(ctx)
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        unstake_handler(ctx)
    }

    pub fn close_staking(ctx: Context<CloseStaking>) -> Result<()> {
        close_staking_handler(ctx)
    }
}

#[error_code]
pub enum StakeError {
    #[msg("unable to get stake details bump")]
    StakeBumpError,
    #[msg("unable to get token authority bump")]
    TokenAuthBumpError,
    #[msg("unable to get token authority bump")]
    NftAuthBumpError,
    #[msg("unable to get nft record bump")]
    NftBumpError,
    #[msg("the minimum staking period in secs can't be negative")]
    NegativePeriodValue,
    #[msg("the given mint account doesn't belong to NFT")]
    TokenNotNFT,
    #[msg("the given token account has no token")]
    TokenAccountEmpty,
    #[msg("the collection field in the metadata is not verified")]
    CollectionNotVerified,
    #[msg("the collection doesn't match the staking details")]
    InvalidCollection,
    #[msg("the minimum stake period for the rewards not completed yet")]
    IneligibleForReward,
    #[msg("the staking is not currently active")]
    StakingInactive,
    #[msg("failed to convert the time to u64")]
    FailedTimeConversion,
    #[msg("unable to add the given values")]
    ProgramAddError,
    #[msg("unable to subtract the given values")]
    ProgramSubError,
    #[msg("unable to multiply the given values")]
    ProgramMulError,
}