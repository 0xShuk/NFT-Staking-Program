use anchor_lang::prelude::*;

#[account]
pub struct NftRecord {
    /// The owner/staker of the NFT (32)
    pub staker: Pubkey,
    /// The mint of the staked NFT (32)
    pub nft_mint: Pubkey,
    /// The staking timestamp (8)
    pub staked_at: i64,
    /// The bump of NFT Record PDA (1)
    pub bump: u8
}

impl NftRecord {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 1;

    pub fn init(staker: Pubkey, nft_mint: Pubkey, staked_at: i64, bump: u8) -> Self {
        Self {staker, nft_mint, staked_at, bump}
    }
}