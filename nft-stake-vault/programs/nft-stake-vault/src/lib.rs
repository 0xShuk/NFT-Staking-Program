use anchor_lang::prelude::*;

mod instructions;
mod state;
mod utils;

use instructions::*;

declare_id!("FZaTXcKpGef7ew74UHpJAkrZAfhMTZbSFJ297aKjURXN");

#[constant]
pub const WEIGHT: u128 = 1_000_000_000;

#[program]
pub mod nft_stake_vault {
    use super::*;

    pub fn init_staking(
        ctx: Context<InitStaking>, 
        reward: u64, 
        minimum_period: i64,
        staking_starts_at: i64,
        staking_ends_at: i64,
        max_stakers_count: u64
    ) -> Result<()> {
        init_staking_handler(ctx, reward, minimum_period, staking_starts_at, staking_ends_at, max_stakers_count)
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

    pub fn extend_staking(ctx: Context<ExtendStaking>, new_end_time: i64) -> Result<()> {
        extend_staking_handler(ctx, new_end_time)
    }

    pub fn change_reward(ctx: Context<ChangeReward>, new_reward: u64) -> Result<()> {
        change_reward_handler(ctx, new_reward)
    }

    pub fn add_funds(ctx: Context<AddFunds>, amount: u64) -> Result<()> {
        add_funds_handler(ctx, amount)
    }

    pub fn close_staking(ctx: Context<CloseStaking>) -> Result<()> {
        close_staking_handler(ctx)
    }
}

#[error_code]
pub enum StakeError {
    #[msg("unable to get stake details bump")]
    StakeBumpError,
    #[msg("unable to get nft record bump")]
    NftBumpError,
    #[msg("the minimum staking period in secs can't be negative")]
    NegativePeriodValue,
    #[msg("stake ends time must be greater than the current time & start time")]
    InvalidStakeEndTime,
    #[msg("the given mint account doesn't belong to NFT")]
    TokenNotNFT,
    #[msg("the given token account has no token")]
    TokenAccountEmpty,
    #[msg("the collection field in the metadata is not verified")]
    CollectionNotVerified,
    #[msg("the collection doesn't match the staking details")]
    InvalidCollection,
    #[msg("max staker count reached")]
    MaxStakersReached,
    #[msg("the minimum stake period for the rewards not completed yet")]
    IneligibleForReward,
    #[msg("the nft stake time is greator than the staking period")]
    StakingIsOver,
    #[msg("the staking is not yet started")]
    StakingNotLive,
    #[msg("the staking is not currently active")]
    StakingInactive,
    #[msg("Insufficient tokens in Vault to extend the period or reward")]
    InsufficientBalInVault,
    #[msg("failed to convert the time to u64")]
    FailedTimeConversion,
    #[msg("failed to convert the weight to u64")]
    FailedWeightConversion,
    #[msg("unable to add the given values")]
    ProgramAddError,
    #[msg("unable to subtract the given values")]
    ProgramSubError,
    #[msg("unable to multiply the given values")]
    ProgramMulError,
    #[msg("unable to divide the given values")]
    ProgramDivError,
}