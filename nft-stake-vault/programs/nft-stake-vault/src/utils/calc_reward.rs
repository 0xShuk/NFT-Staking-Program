use anchor_lang::prelude::*;
use crate::StakeError;

pub fn calc_reward(
    staked_at: i64,
    minimum_stake_period: i64,
    reward_emission: &Vec<u64>,
    reward_change_time: &Vec<i64>,
    staking_ends_at: i64
) -> Result<(u64, i64, bool)> {
    let clock = Clock::get().unwrap();
    let current_time = clock.unix_timestamp;

    let reward_eligible_time = staked_at.checked_add(minimum_stake_period).ok_or(StakeError::ProgramAddError)?;
    let is_eligible_for_reward = current_time >= reward_eligible_time;

    let cutoff_time = i64::min(current_time, staking_ends_at);
    
    // The index during which NFT staked
    let stake_index = reward_change_time.binary_search(&staked_at);

    let index = match stake_index {
        Ok(i) => i,
        Err(i) => i - 1
    };

    let mut reward_tokens: u64 = 0;
    let total_changes = reward_change_time.len() - 1;

    // Going through every reward change between NFT staked and reward claimed
    for ix in index..=total_changes {
        let big_num = if ix == total_changes { cutoff_time } else { reward_change_time[ix + 1] };
        let sml_num = if ix == index { staked_at } else { reward_change_time[ix] };

        let rewardable_time = big_num.checked_sub(sml_num).ok_or(StakeError::ProgramSubError)?;

        let rewardable_time = match u64::try_from(rewardable_time) {
            Ok(time) => time,
            _ => return err!(StakeError::FailedTimeConversion)
        };

        let reward = rewardable_time.checked_mul(reward_emission[ix]).ok_or(StakeError::ProgramMulError)?;

        reward_tokens = reward_tokens.checked_add(reward).ok_or(StakeError::ProgramAddError)?;
    }

    Ok((reward_tokens, current_time, is_eligible_for_reward))
}
