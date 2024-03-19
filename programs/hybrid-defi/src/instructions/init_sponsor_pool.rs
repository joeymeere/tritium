use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::state::Sponsor;

pub fn init_sponsor_pool(
    ctx: Context<InitSponsor>,
    nft_mint: Pubkey,
    token_mint: Pubkey,
    swap_factor: u64,
) -> Result<()> {
    ctx.accounts.hybrid_vault.set_inner(
        Sponsor::new(
            ctx.accounts.payer.key(), 
            nft_mint, 
            token_mint, 
            swap_factor,
            ctx.bumps.nft_authority,
            ctx.bumps.hybrid_vault,
        )?
    );

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    nft_mint: Pubkey,
    token_mint: Pubkey,
    swap_factor: u64,
)]
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
        space = 86
    )]
    pub hybrid_vault: Account<'info, Sponsor>,

    #[account(mint::decimals = 0)]
    pub collection_mint: Account<'info, Mint>,

    /// CHECK: This account is not read or written
    #[account(
        seeds = [b"nft_authority", hybrid_vault.key().as_ref()],
        bump
    )]
    pub nft_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}