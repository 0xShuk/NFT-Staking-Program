use anchor_lang::prelude::*;

use crate::{state::Details, utils::{calc_total_emission, calc_actual_balance}, StakeError};

#[derive(Accounts)]
pub struct ChangeReward<'info> {
    #[account(
        mut,
        seeds = [
            b"stake", 
            stake_details.collection.as_ref(),
            stake_details.creator.as_ref()
        ],
        bump = stake_details.stake_bump,
        has_one = creator,
        realloc = stake_details.current_len() + 16,
        realloc::payer = creator,
        realloc::zero = false
    )]
    pub stake_details: Account<'info, Details>,

    #[account(mut)]
    pub creator: Signer<'info>,

    pub system_program: Program<'info, System>
}

pub fn change_reward_handler(ctx: Context<ChangeReward>, new_reward: u64) -> Result<()> {
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

    require_gte!(staking_ends_at, current_time, StakeError::StakingIsOver);
    require_eq!(staking_status, true, StakeError::StakingInactive);

    let (current_actual_balance, new_staked_weight) = calc_actual_balance(
        current_stakers_count,
        staked_weight,
        current_reward,
        last_reward_change_time,
        staking_ends_at,
        current_time,
        current_balance,
        None
    )?;

    let new_emission = calc_total_emission(
        new_reward, 
        max_stakers_count, 
        current_time, 
        staking_ends_at
    )?;

    require_gte!(current_actual_balance, new_emission, StakeError::InsufficientBalInVault);

    let stake_details = &mut ctx.accounts.stake_details;

    stake_details.change_reward(new_reward, current_time);
    stake_details.current_balance = current_actual_balance;
    stake_details.staked_weight = new_staked_weight;

    Ok(())
}