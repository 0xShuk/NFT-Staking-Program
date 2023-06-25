use anchor_lang::prelude::*;
use crate::{StakeError, WEIGHT};

pub fn calc_actual_balance(
    current_stakers_count: u64,
    staked_weight: u128,
    last_reward_rate: u64,
    last_reward_time: i64,
    staking_ends_at: i64,
    current_time: i64,
    current_balance: u64,
    new_end_time: Option<i64>
) -> Result<(u64, u128)> {
    let avg_staked_weight = if staked_weight == 0 {
        staked_weight
    } else {
        staked_weight
        .checked_div(current_stakers_count as u128)
        .ok_or(StakeError::ProgramDivError)? + 1
    };

    // Total time since last reward change to stake end
    let total_time = staking_ends_at
        .checked_sub(last_reward_time)
        .ok_or(StakeError::ProgramSubError)?;

    let total_time_u128 = match u128::try_from(total_time) {
        Ok(time) => time,
        _ => return err!(StakeError::FailedTimeConversion)
    };

    // Time between average staking time and stake end
    let stake_to_end_time_weighted = total_time_u128
        .checked_mul(avg_staked_weight)
        .ok_or(StakeError::ProgramMulError)?;

    let stake_to_end_time = stake_to_end_time_weighted
        .checked_div(WEIGHT)
        .ok_or(StakeError::ProgramDivError)? + 1;

    let stake_to_end_time = match u64::try_from(stake_to_end_time) {
        Ok(time) => time,
        _ => return err!(StakeError::FailedTimeConversion)
    };

    // Calculate Rewardable Time
    let rewardable_time = if staking_ends_at > current_time {
        // If the current time is less than the stake end time,
        // Subtract the unaccrued time from the stake to end time
        let unaccrued_time = staking_ends_at
            .checked_sub(current_time)
            .ok_or(StakeError::ProgramSubError)?;

        let unaccrued_time_u64 = match u64::try_from(unaccrued_time) {
            Ok(time) => time,
            _ => return err!(StakeError::FailedTimeConversion)
        };

        stake_to_end_time
        .checked_sub(unaccrued_time_u64)
        .ok_or(StakeError::ProgramSubError)?
    } else {
        // If the current time is greater or equal to the stake end time,
        // add seconds since the stake end time to the rewardable time
        let accrued_time = current_time
            .checked_sub(staking_ends_at)
            .ok_or(StakeError::ProgramSubError)?;

        let accrued_time_u64 = match u64::try_from(accrued_time) {
            Ok(time) => time,
            _ => return err!(StakeError::FailedTimeConversion)
        };

        stake_to_end_time
        .checked_add(accrued_time_u64)
        .ok_or(StakeError::ProgramAddError)?
    };

    // The rewards yet to be paid (per staker)
    let accrued_reward = last_reward_rate
        .checked_mul(rewardable_time)
        .ok_or(StakeError::ProgramMulError)?;

    // The rewards yet to be paid (all stakers)
    let accrued_reward = accrued_reward
        .checked_mul(current_stakers_count)
        .ok_or(StakeError::ProgramMulError)?;

    // The current actual balance after deducting accrual rewards
    let current_actual_balance = current_balance
        .checked_sub(accrued_reward)
        .ok_or(StakeError::ProgramSubError)?;

    // THE CALCULATION OF THE NEW STAKED WEIGHT
    let new_staked_weight = match new_end_time {
        Some(new_time) => {
            let stake_to_old_end = match i64::try_from(stake_to_end_time) {
                Ok(stake_to_end) => stake_to_end,
                _ => return err!(StakeError::FailedTimeConversion)
            };

            let time_added = new_time
                .checked_sub(staking_ends_at)
                .ok_or(StakeError::ProgramSubError
            )?;

            // Add extended time to stake period
            let stake_to_new_end = stake_to_old_end
                .checked_add(time_added)
                .ok_or(StakeError::ProgramAddError
            )?;

            let new_base = new_time
                .checked_sub(last_reward_time)
                .ok_or(StakeError::ProgramSubError
            )?;

            let stake_to_new_end_u128 = match u128::try_from(stake_to_new_end) {
                Ok(stake_to_end) => stake_to_end,
                _ => return err!(StakeError::FailedTimeConversion)
            };

            let new_base_u128 = match u128::try_from(new_base) {
                Ok(base) => base,
                _ => return err!(StakeError::FailedTimeConversion)
            };

            let new_num = stake_to_new_end_u128.checked_mul(WEIGHT).ok_or(StakeError::ProgramMulError)?;

            // New average staked weight
            let new_weight = new_num.checked_div(new_base_u128).ok_or(StakeError::ProgramDivError)?;

            // New total staked weight
            new_weight.checked_mul(current_stakers_count as u128).ok_or(StakeError::ProgramMulError)?
        },
        None => {
            // Return the whole weight if reward is changed
            WEIGHT.checked_mul(current_stakers_count as u128).ok_or(StakeError::ProgramMulError)?
        }
    };

    Ok((current_actual_balance, new_staked_weight))
}