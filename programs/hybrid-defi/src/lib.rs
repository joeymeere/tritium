use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod util;
pub mod error;

use crate::instructions::*;
use crate::state::*;
use crate::util::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod hybrid_defi {
    use super::*;

    pub fn initialize_sponsor_pool(ctx: Context<InitSponsor>, nft_mint: Pubkey, token_mint: Pubkey, swap_factor: u64) -> Result<()> {
        instructions::init_sponsor_pool(ctx, nft_mint, token_mint, swap_factor)
    }

    pub fn swap_nft_to_token(ctx: Context<SwapNFTToToken>) -> Result<()> {
        instructions::swap_nft_to_token(ctx)
    }
}
