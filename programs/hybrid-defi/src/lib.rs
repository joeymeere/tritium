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

    pub fn initialize_sponsor_pool(
        ctx: Context<InitSponsor>, 
        swap_factor: u64, 
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
}
