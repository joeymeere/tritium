use anchor_lang::prelude::*;
use anchor_spl::{associated_token, token::{self, Mint}};

use crate::state::Sponsor;

// Creates a "sponsor" account, and deposits 
// initial tokens into the vault.
pub fn init_sponsor_pool(
    ctx: Context<InitSponsor>,
    swap_factor: u64,
    initial_balance: u64,
    lamport_fee: u64,
) -> Result<()> {
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

    // Transfer initial tokens to vault
    // "initial_balance" arg should account for decimals
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.payer_token_account.to_account_info(),
                to: ctx.accounts.sponsor_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        initial_balance,
    )?;

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
        seeds = [
            b"nft_authority", 
            hybrid_vault.key().as_ref()
        ],
        bump
    )]
    pub nft_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub token_mint: Account<'info, token::Mint>,
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = hybrid_vault,
    )]
    pub sponsor_token_account: Account<'info, token::TokenAccount>,
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = payer,
    )]
    pub payer_token_account: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}