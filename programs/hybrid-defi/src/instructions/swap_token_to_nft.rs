use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_lang::solana_program::sysvar;
use anchor_spl::token;
use anchor_spl::{
    associated_token::AssociatedToken, 
    metadata::{mpl_token_metadata::instructions::TransferV1CpiBuilder, MasterEditionAccount, Metadata, MetadataAccount, TokenRecordAccount}, 
    token::{Mint, Token, TokenAccount}
};

use crate::error::HybridErrorCode;
use crate::Sponsor;

pub fn swap_token_to_nft(ctx: Context<SwapTokenToNFT>, amount: u64) -> Result<()> {
    let sponsor = &mut ctx.accounts.sponsor;
        sponsor.nfts_held = sponsor.nfts_held.checked_sub(1).unwrap();

        require!(amount == sponsor.swap_factor, HybridErrorCode::InsufficientTokens);

        let mut transfer_cpi = TransferV1CpiBuilder::new(&ctx.accounts.metadata_program);

        let nft_token = &ctx.accounts.nft_token.to_account_info();
        let payer = &ctx.accounts.payer.to_account_info();
        let nft_custody = &ctx.accounts.nft_custody.to_account_info();
        let nft_mint = &ctx.accounts.nft_mint.to_account_info();
        let nft_metadata = &ctx.accounts.nft_metadata.to_account_info();
        let nft_edition = &ctx.accounts.nft_edition.to_account_info();
        let source_token_record = &ctx.accounts.source_token_record.to_account_info();
        let destination_token_record = &ctx.accounts.destination_token_record.to_account_info();
        // let auth_rules_program = &ctx.accounts.auth_rules_program.to_account_info();
        // let auth_rules = &ctx.accounts.auth_rules.to_account_info();
        
        transfer_cpi
        .token(nft_custody)
        .token_owner(&ctx.accounts.nft_authority)
        .destination_token(nft_token)
        .destination_owner(payer)
        .mint(nft_mint)
        .metadata(nft_metadata)
        .edition(Some(nft_edition))
        .token_record(Some(source_token_record))
        .destination_token_record(Some(destination_token_record))
        .authority(&ctx.accounts.nft_authority)
        .payer(payer)
        .system_program(&ctx.accounts.system_program)
        .sysvar_instructions(&ctx.accounts.sysvar_instructions)
        .spl_token_program(&ctx.accounts.token_program)
        .spl_ata_program(&ctx.accounts.associated_token_program)
        // .authorization_rules_program(None)
        // .authorization_rules(None)
        .amount(1);

        const PREFIX_SEED: &'static [u8] = b"nft_authority";
        let signer_seeds = [PREFIX_SEED, &sponsor.key().to_bytes(), &[sponsor.auth_rules_bump]];

        transfer_cpi.invoke_signed(&[&signer_seeds])?;

        anchor_spl::token::close_account(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            anchor_spl::token::CloseAccount {
                account: nft_custody.to_account_info(),
                destination: payer.to_account_info(),
                authority: ctx.accounts.nft_authority.to_account_info()
            },
            &[&signer_seeds]
        ))?;

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.payer_token_account.to_account_info(),
                    to: ctx.accounts.sponsor_token_account.to_account_info(),
                    authority: sponsor.to_account_info(),
                },
                &[&signer_seeds]
            ),
            amount,
        )?;

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.fee_wallet.to_account_info(),
                },
            ),
            3000000,
        )?;

    Ok(())
}

#[derive(Accounts)]
pub struct SwapTokenToNFT<'info> {
    #[account(
        mut, 
        seeds = [
            b"hybrid_sponsor", 
            sponsor.authority.key().as_ref(),
            sponsor.nft_mint.key().as_ref(), 
        ], 
        bump = sponsor.bump
    )]
    pub sponsor: Account<'info, Sponsor>,

    #[account(
        mut,
        token::mint = sponsor.token_mint,
        token::authority = sponsor,
    )]
    pub sponsor_token_account: Account<'info, token::TokenAccount>,
    #[account(
        mut,
        token::mint = sponsor.token_mint,
        token::authority = payer,
    )]
    pub payer_token_account: Account<'info, token::TokenAccount>,

    #[account(
        mint::decimals = 0, 
        constraint = nft_mint.supply == 1
    )]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut, 
        associated_token::mint = nft_mint, 
        associated_token::authority = payer, 
        constraint = nft_token.amount == 0
    )]
    pub nft_token: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            b"metadata", 
            Metadata::id().as_ref(), 
            nft_mint.key().as_ref()
        ],
        seeds::program = Metadata::id(),
        bump,
        constraint = nft_metadata.collection.as_ref().unwrap().verified,
        constraint = nft_metadata.collection.as_ref().unwrap().key == sponsor.nft_mint
    )]
    pub nft_metadata: Box<Account<'info, MetadataAccount>>,

    #[account(
        seeds = [b"metadata",
            Metadata::id().as_ref(),
            nft_mint.key().as_ref(),
            b"edition"
        ],
        seeds::program = Metadata::id(),
        bump
    )]
    pub nft_edition: Box<Account<'info, MasterEditionAccount>>,

    /// CHECK: This account is not read or written
    #[account(
        mut, 
        seeds = [
            b"nft_authority", 
            sponsor.key().as_ref()
        ], 
        bump = sponsor.auth_rules_bump
    )]
    pub nft_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = nft_mint, 
        associated_token::authority = nft_authority
    )]
    pub nft_custody: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            b"metadata", 
            Metadata::id().as_ref(),
            nft_mint.key().as_ref(),
            b"token_record",
            nft_custody.key().as_ref(),
        ],
        seeds::program = Metadata::id(),
        bump
    )]
    pub source_token_record: Account<'info, TokenRecordAccount>,

    /// CHECK: account constraints checked in account trait
    #[account(
        mut,
        seeds = [
            b"metadata", 
            Metadata::id().as_ref(),
            nft_mint.key().as_ref(),
            b"token_record",
            nft_token.key().as_ref(),
        ],
        seeds::program = Metadata::id(),
        bump
    )]
    pub destination_token_record: UncheckedAccount<'info>,

    #[account(
        mut, 
        constraint = payer.key() == sponsor.authority
    )]
    pub payer: Signer<'info>,

    #[account(
        mut
    )]
    pub fee_wallet: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

    /// CHECK: account constraints checked in account trait
    #[account(
        address = sysvar::instructions::id()
    )]
    pub sysvar_instructions: UncheckedAccount<'info>,
}