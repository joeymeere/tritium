use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod util;
pub mod error;

use crate::instructions::*;
use crate::state::*;

declare_id!("AoFwcQnifpNDKrZeaTCDBNuvciXq5parfUJuhvojd5x7");

#[program]
pub mod tritium {
    use super::*;

    // Initializes the Sponsor PDA account
    pub fn initialize_sponsor_pool(
        ctx: Context<InitSponsor>, 
        name: String,
        swap_factor: [f64; 3], 
        lamport_fee: u64,
    ) -> Result<()> {
        instructions::init_sponsor_pool(
            ctx, 
            name,
            swap_factor, 
            lamport_fee
        )
    }

    // Deposits initial and future SPL tokens into Sponsor
    pub fn deposit_tokens(
        ctx: Context<DepositTokens>,
        amount: u64
    ) -> Result<()> {
        instructions::deposit_tokens(
            ctx, 
            amount
        )
    }

    // Swaps pNFT to SPL based on "swap_factor"
    pub fn swap_nft_to_token(
        ctx: Context<SwapNFTToToken>
    ) -> Result<()> {
        instructions::swap_nft_to_token(ctx)
    }

    // Swaps SPL to pNFT based on "swap_factor[0]"
    pub fn swap_token_to_nft(
        ctx: Context<SwapTokenToNFT>,
        //amount: f64
    ) -> Result<()> {
        instructions::swap_token_to_nft(ctx)
    }
}
