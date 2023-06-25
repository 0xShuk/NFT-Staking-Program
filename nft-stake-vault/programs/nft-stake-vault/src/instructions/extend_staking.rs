use anchor_lang::prelude::*;

use crate::{state::Details, utils::{calc_total_emission, calc_actual_balance}, StakeError};

#[derive(Accounts)]
pub struct ExtendStaking<'info> {
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

    pub creator: Signer<'info>,
}

pub fn extend_staking_handler(ctx: Context<ExtendStaking>, new_ending_time: i64) -> Result<()> {
    let stake_details = &ctx.accounts.stake_details;
    let current_time = Clock::get().unwrap().unix_timestamp;

    let Details {
        max_stakers_count,
        current_stakers_count,
        staking_ends_at,
        current_balance,
        staked_weight,
        is_active: staking_status,
        ..
    } = **stake_details;

    let current_reward = *stake_details.reward.last().unwrap();
    let last_reward_change_time = *stake_details.reward_change_time.last().unwrap();

    require_eq!(staking_status, true, StakeError::StakingInactive);
    require_gt!(new_ending_time, current_time, StakeError::InvalidStakeEndTime);
    require_gt!(new_ending_time, staking_ends_at, StakeError::InvalidStakeEndTime);
    
    let (current_actual_balance, new_staked_weight) = calc_actual_balance(
        current_stakers_count,
        staked_weight,
        current_reward,
        last_reward_change_time,
        staking_ends_at,
        current_time,
        current_balance,
        Some(new_ending_time)
    )?;

    let new_emission = calc_total_emission(
        current_reward, 
        max_stakers_count, 
        current_time, 
        new_ending_time
    )?;

    require_gte!(current_actual_balance, new_emission, StakeError::InsufficientBalInVault);

    let stake_details = &mut ctx.accounts.stake_details;

    stake_details.extend_staking(new_ending_time);
    stake_details.staked_weight = new_staked_weight;

    Ok(())
}