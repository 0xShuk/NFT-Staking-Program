use anchor_lang::prelude::*;


#[account]
pub struct Details {
    /// The status of the staking (1)
    pub is_active: bool,
    /// The creator of the stake record (32)
    pub creator: Pubkey,
    /// The mint of the token to be given as reward (32)
    pub reward_mint: Pubkey,
    /// The rate of reward emission per second (8)
    pub reward: u64,
    /// The verified collection address of the NFT (32)
    pub collection: Pubkey,
    /// The minimum stake period to be eligible for reward - in seconds (8)
    pub minimum_period: i64,
    /// The bump of stake details PDA (1)
    pub stake_bump: u8,
    /// The bump of token authority PDA (1)
    pub token_auth_bump: u8,
    /// The bump of nft authority PDA (1)
    pub nft_auth_bump: u8
}

impl Details {
    pub const LEN: usize = 8 + 1 + 32 + 32 + 8 + 32 + 8 + 1 + 1 + 1;

    pub fn init(
        creator: Pubkey,
        reward_mint: Pubkey,
        reward: u64,
        collection: Pubkey,
        minimum_period: i64,
        stake_bump: u8,
        token_auth_bump: u8,
        nft_auth_bump: u8
    ) -> Self {
        Self {
            is_active: true,
            creator,
            reward_mint,
            reward,
            collection,
            minimum_period,
            stake_bump,
            token_auth_bump,
            nft_auth_bump
        }
    }

    pub fn close_staking(&mut self) -> Result<()> {
        self.is_active = false;
        Ok(())
    }
}