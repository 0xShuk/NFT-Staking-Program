use anchor_lang::prelude::*;
use crate::StakeError;

pub fn calc_total_emission(
    reward: u64,
    max_stakers_count: u64,
    staking_starts_at: i64,
    staking_ends_at: i64
) -> Result<u64> {
    let total_staking_period = staking_ends_at.checked_sub(staking_starts_at).ok_or(StakeError::ProgramSubError)?;

    let rewardable_time_u64 = match u64::try_from(total_staking_period) {
        Ok(time) => time,
        _ => return err!(StakeError::FailedTimeConversion)
    };

    let total_rewardable_time = rewardable_time_u64.checked_mul(max_stakers_count).ok_or(StakeError::ProgramMulError)?;
    let total_emission = total_rewardable_time.checked_mul(reward).ok_or(StakeError::ProgramMulError)?;

    Ok(total_emission)
}
