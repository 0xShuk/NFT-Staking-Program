use anchor_lang::prelude::*;

use crate::{StakeError, WEIGHT};

#[account]
pub struct Details {
    /// The status of the staking (1)
    pub is_active: bool,
    /// The creator of the stake record (32)
    pub creator: Pubkey,
    /// The mint of the token to be given as reward (32)
    pub reward_mint: Pubkey,
    /// The record of the current and prev reward emissions
    pub reward: Vec<u64>,
    /// the record of the time when reward emission changed
    pub reward_change_time: Vec<i64>,
    /// The verified collection address of the NFT (32)
    pub collection: Pubkey,
    /// The max number of NFTs that can be staked (8)
    pub max_stakers_count: u64,
    /// The current number of NFTs staked (8)
    pub current_stakers_count: u64,
    /// Accrued weight of the staked NFTs (16)
    pub staked_weight: u128, 
    /// The starting time of the staking (8)
    pub staking_starts_at: i64,
    /// The period for which staking is funded (8)
    pub staking_ends_at: i64,
    /// The minimum stake period to be eligible for reward - in seconds (8)
    pub minimum_period: i64,
    /// The bump of the stake record PDA (1)
    pub stake_bump: u8,
    /// The bump of the token authority PDA (1)
    pub token_auth_bump: u8,
    /// The bump of the nft authority PDA (1)
    pub nft_auth_bump: u8,
    /// The current balance in Stake Vault (8)
    pub current_balance: u64
}

impl Details {
    pub const LEN: usize = 8 + 1 + 32 + 32 + 12 + 12 + 32 + 8 + 8 + 16 + 8 + 8 + 8 + 1 + 1 + 1 + 8;

    pub fn init(
        creator: Pubkey,
        reward_mint: Pubkey,
        collection: Pubkey,
        reward: u64,
        max_stakers_count: u64,
        staking_starts_at: i64,
        staking_ends_at: i64,
        minimum_period: i64,
        stake_bump: u8,
        token_auth_bump: u8,
        nft_auth_bump: u8,
        current_balance: u64
    ) -> Self {
        Self {
            is_active: true,
            creator,
            reward_mint,
            collection,
            reward: vec![reward],
            reward_change_time: vec![staking_starts_at],
            max_stakers_count,
            staked_weight: 0,
            current_stakers_count: 0,
            staking_starts_at,
            staking_ends_at,
            minimum_period,
            stake_bump,
            token_auth_bump,
            nft_auth_bump,
            current_balance
        }
    }

    pub fn current_len(&self) -> usize {
        (Details::LEN - 16) + (self.reward.len() * 16)
    }

    pub fn change_reward(&mut self, new_reward: u64, current_time: i64) {
        self.reward.push(new_reward);
        self.reward_change_time.push(current_time);
    }

    pub fn extend_staking(&mut self, new_end_time: i64) {
        self.staking_ends_at = new_end_time;
    }

    pub fn update_staked_weight(&mut self, stake_time: i64, increase_weight: bool) -> Result<()> {
        let last_reward_time = *self.reward_change_time.last().unwrap();

        let base = self.staking_ends_at
            .checked_sub(last_reward_time)
            .ok_or(StakeError::ProgramSubError)? as u128; // directly converting to u128 since it can't be negative

        let weight_time = stake_time.max(last_reward_time);

        let mut num = self.staking_ends_at
            .checked_sub(weight_time)
            .ok_or(StakeError::ProgramSubError)? as u128; // directly converting to u128 since it can't be negative

        num = num.checked_mul(WEIGHT).ok_or(StakeError::ProgramMulError)?;
        
        let weight = num.checked_div(base).ok_or(StakeError::ProgramDivError)?;

        if increase_weight {
            self.staked_weight = self.staked_weight.checked_add(weight).ok_or(StakeError::ProgramAddError)?;
        } else {
            self.staked_weight = self.staked_weight.checked_sub(weight).ok_or(StakeError::ProgramSubError)?;
        }

        Ok(())
    }

    pub fn increase_staker_count(&mut self) -> Result<()> {
        self.current_stakers_count = self.current_stakers_count
        .checked_add(1)
        .ok_or(StakeError::ProgramAddError)?;
        
        Ok(())
    }

    pub fn decrease_staker_count(&mut self) -> Result<()> {
        self.current_stakers_count = self.current_stakers_count
        .checked_sub(1)
        .ok_or(StakeError::ProgramSubError)?;
        
        Ok(())
    }
    
    pub fn increase_current_balance(&mut self, added_funds: u64) -> Result<()> {
        self.current_balance = self.current_balance
            .checked_add(added_funds)
            .ok_or(StakeError::ProgramAddError)?;
        
        Ok(())
    }

    pub fn decrease_current_balance(&mut self, staked_at: i64, current_time: i64) -> Result<()> {
        let last_reward_time = *self.reward_change_time.last().unwrap();
        let last_reward = *self.reward.last().unwrap();

        let reward_time = staked_at.max(last_reward_time);
        let cutoff_time = current_time.min(self.staking_ends_at);

        let rewardable_time_since_change = cutoff_time
            .checked_sub(reward_time)
            .ok_or(StakeError::ProgramSubError)?;

        let rewardable_time_u64 = match u64::try_from(rewardable_time_since_change) {
            Ok(time) => time,
            _ => return err!(StakeError::FailedTimeConversion)
        };

        let reward_since_change = last_reward
            .checked_mul(rewardable_time_u64)
            .ok_or(StakeError::ProgramMulError)?;

        self.current_balance = self.current_balance
            .checked_sub(reward_since_change)
            .ok_or(StakeError::ProgramSubError)?;
        
        Ok(())
    }

    pub fn close_staking(&mut self) {
        self.is_active = false;
    }
}