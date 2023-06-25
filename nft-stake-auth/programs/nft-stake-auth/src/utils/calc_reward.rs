use anchor_lang::prelude::*;
use crate::StakeError;

pub fn calc_reward(
    staked_at: i64,
    minimum_stake_period: i64,
    reward_emission: u64,
) -> Result<(u64, i64, bool)> {
    let clock = Clock::get().unwrap();
    let current_time = clock.unix_timestamp;

    let reward_eligible_time = staked_at.checked_add(minimum_stake_period).ok_or(StakeError::ProgramAddError)?;
    let is_eligible_for_reward = current_time >= reward_eligible_time;

    let rewardable_time_i64 = current_time.checked_sub(staked_at).ok_or(StakeError::ProgramSubError)?;

    let rewardable_time_u64 = match u64::try_from(rewardable_time_i64) {
        Ok(time) => time,
        _ => return err!(StakeError::FailedTimeConversion)
    };

    let reward_tokens = rewardable_time_u64.checked_mul(reward_emission).ok_or(StakeError::ProgramMulError)?;
    Ok((reward_tokens, current_time, is_eligible_for_reward))
}
