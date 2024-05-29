use anchor_lang::prelude::*;

#[account]
pub struct Sponsor {
    pub name: String,
    pub authority: Pubkey,
    pub nft_mint: Pubkey,
    pub token_mint: Pubkey,
    pub nfts_held: u64,
    pub swap_factor: [f64; 3], // [ 1 (Baseline), 2 (Rare Multiplier), 3 (Legendary Multiplier) ]
    pub auth_rules_bump: u8,
    pub bump: u8,
    pub lamport_fee: u64,
    pub withdrawable: bool,
}

impl Sponsor {
    pub const SEED_PREFIX: &'static str = "hybrid_defi";
    pub const SPACE: usize = 8 
        + 32                       // u64
        + 32                       // String
        + 32                       // u64
        + 4                        // u64
        + 24                       // [f64; 3]
        + 1                        // u8
        + 1                        // u8
        + 4                        // u8
        + 650;                     // Padding
    
    pub fn new(
        name: String,
        authority: Pubkey, 
        nft_mint: Pubkey, 
        token_mint: Pubkey, 
        swap_factor: [f64; 3], 
        auth_rules_bump: u8, 
        bump: u8, 
        lamport_fee: u64
    ) -> Result<Self> {
        Ok(Self {
            name,
            authority,
            nft_mint,
            token_mint,
            nfts_held: 0,
            swap_factor,
            auth_rules_bump,
            bump,
            lamport_fee,
            withdrawable: true
        })
    }
}