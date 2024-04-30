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

    pub fn initialize_sponsor_pool(
        ctx: Context<InitSponsor>, 
        swap_factor: [f64; 3], 
        initial_balance: u64,
        lamport_fee: u64,
    ) -> Result<()> {
        instructions::init_sponsor_pool(
            ctx, 
            swap_factor, 
            initial_balance, 
            lamport_fee
        )
    }

    pub fn swap_nft_to_token(
        ctx: Context<SwapNFTToToken>
    ) -> Result<()> {
        instructions::swap_nft_to_token(ctx)
    }

    pub fn swap_token_to_nft(
        ctx: Context<SwapTokenToNFT>,
        amount: f64
    ) -> Result<()> {
        instructions::swap_token_to_nft(ctx, amount)
    }
}
