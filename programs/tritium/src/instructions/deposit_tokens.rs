use anchor_lang::prelude::*;
use anchor_spl::{associated_token, token::{self, Mint}};

use crate::{state::Sponsor, util::WL_KEYS};
use crate::error::HybridErrorCode;

// This instruction is for depositing SPL tokens into the pool
// Should only be called by an admin, as no NFT is returned from this.
pub fn deposit_tokens(
    ctx: Context<DepositTokens>,
    amount: u64,
) -> Result<()> {
    /*
    require!(
        WL_KEYS.contains(&ctx.accounts.payer.key().to_string().as_str()), 
        HybridErrorCode::UnauthorizedCreation
    );
    */

    // Transfer initial tokens to vault
    // "amount" arg should account for decimals
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.payer_token_account.to_account_info(),
                to: ctx.accounts.sponsor_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        amount,
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    amount: u64,
)]
pub struct DepositTokens<'info> {
    #[account(
        mut,
        seeds = [
            b"hybrid_sponsor", 
            hybrid_vault.authority.as_ref(),
            hybrid_vault.nft_mint.key().as_ref(), 
            hybrid_vault.name.as_ref(),
        ],
        bump,
    )]
    pub hybrid_vault: Account<'info, Sponsor>,
    #[account(
        mint::decimals = 0,
        constraint = collection_mint.key() == hybrid_vault.nft_mint
    )]
    pub collection_mint: Account<'info, Mint>,
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