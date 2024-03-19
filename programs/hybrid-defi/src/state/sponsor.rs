use anchor_lang::prelude::*;

#[account]
pub struct Sponsor {
    pub authority: Pubkey,
    pub nft_mint: Pubkey,
    pub token_mint: Pubkey,
    pub nfts_held: u64,
    pub swap_factor: u64,
    pub auth_rules_bump: u8,
    pub bump: u8
}

impl Sponsor {
    pub const SEED_PREFIX: &'static str = "fundraiser";

    pub const SPACE: usize = 8 
        + 4                         // u64
        + 4                         // String
        + 4                         // u64
        + 4                         // u64
        + 4                         // u64
        + 1                         // u8
        + 160                       // Vec<Pubkey> (Max 5)
        + 32                        // Pubkey
        + 1                         // u8
        + 4                         // Enum (Singleton)
        + 250;                      // Padding
    
    pub fn new(authority: Pubkey, nft_mint: Pubkey, token_mint: Pubkey, swap_factor: u64, auth_rules_bump: u8, bump: u8) -> Result<Self> {
        Ok(Self {
            authority,
            nft_mint,
            token_mint,
            nfts_held: 0,
            swap_factor,
            auth_rules_bump,
            bump
        })
    }
}