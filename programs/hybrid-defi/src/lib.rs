use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod util;
pub mod error;

use crate::instructions::*;
use crate::state::*;

declare_id!("B5KWHdNtiXtc1gS8aLobWS8hP3Xvdgi618VL81y8cw6V");

#[program]
pub mod hybrid_defi {
    use super::*;

    // Initializes the Sponsor PDA account
    pub fn initialize_sponsor_pool(
        ctx: Context<InitSponsor>, 
        swap_factor: [f64; 3], 
        lamport_fee: u64,
    ) -> Result<()> {
        instructions::init_sponsor_pool(
            ctx, 
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
        amount: f64
    ) -> Result<()> {
        instructions::swap_token_to_nft(ctx, amount)
    }
}
