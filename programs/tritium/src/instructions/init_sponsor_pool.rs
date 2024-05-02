use anchor_lang::prelude::*;
use anchor_spl::{token::{self, Mint}};

use crate::{state::Sponsor, util::WL_KEYS};
use crate::error::HybridErrorCode;

// Creates a "sponsor" account, and deposits 
// initial tokens into the vault.
pub fn init_sponsor_pool(
    ctx: Context<InitSponsor>,
    swap_factor: [f64; 3],
    lamport_fee: u64,
) -> Result<()> {
    /*
    require!(
        WL_KEYS.contains(&ctx.accounts.payer.key().to_string().as_str()), 
        HybridErrorCode::UnauthorizedCreation
    );
    */

    ctx.accounts.hybrid_vault.set_inner(
        Sponsor::new(
            ctx.accounts.payer.key(), 
            ctx.accounts.collection_mint.key(),
            ctx.accounts.token_mint.key(),
            swap_factor,
            ctx.bumps.nft_authority,
            ctx.bumps.hybrid_vault,
            lamport_fee,
        )?
    );

    Ok(())
}

#[derive(Accounts)]
pub struct InitSponsor<'info> {
    #[account(
        init,
        payer = payer,
        seeds = [
            b"hybrid_sponsor", 
            payer.key().as_ref(),
            collection_mint.key().as_ref(), 
        ],
        bump,
        space = Sponsor::SPACE
    )]
    pub hybrid_vault: Account<'info, Sponsor>,
    #[account(mint::decimals = 0)]
    pub collection_mint: Account<'info, Mint>,
    /// CHECK: This account is not read or written
    #[account(
        seeds = [
            b"nft_authority", 
            hybrid_vault.key().as_ref()
        ],
        bump
    )]
    pub nft_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub token_mint: Account<'info, token::Mint>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}