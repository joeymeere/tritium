use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use anchor_spl::{
    token::{Mint, TokenAccount, Token}, 
    metadata::{MetadataAccount, Metadata, MasterEditionAccount, mpl_token_metadata::instructions::TransferV1CpiBuilder, TokenRecordAccount},
    associated_token::AssociatedToken,
};

use crate::Sponsor;

pub fn swap_nft_to_token<'info>(
    ctx: Context<SwapNFTToToken>,
) -> Result<()> {
    let sponsor = &mut ctx.accounts.sponsor;
        sponsor.nfts_held = sponsor.nfts_held.checked_add(1).unwrap();

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
        .token(nft_token)
        .token_owner(payer)
        .destination_token(nft_custody)
        .destination_owner(&ctx.accounts.nft_authority)
        .mint(nft_mint)
        .metadata(nft_metadata)
        .edition(Some(nft_edition))
        .token_record(Some(source_token_record))
        .destination_token_record(Some(destination_token_record))
        .authority(payer)
        .payer(payer)
        .system_program(&ctx.accounts.system_program)
        .sysvar_instructions(&ctx.accounts.sysvar_instructions)
        .spl_token_program(&ctx.accounts.token_program)
        .spl_ata_program(&ctx.accounts.associated_token_program)
        // .authorization_rules_program(None)
        // .authorization_rules(None)
        .amount(1);

        transfer_cpi.invoke()?;

    Ok(())
}

#[derive(Accounts)]
pub struct SwapNFTToToken<'info> {
    #[account(
        mut, 
        seeds = [
            b"hybrid_defi", 
            sponsor.authority.as_ref(),
            sponsor.nft_mint.as_ref()
        ], 
        bump = sponsor.bump
    )]
    pub sponsor: Account<'info, Sponsor>,

    #[account(mint::decimals = 0, constraint = nft_mint.supply == 1)]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut, 
        associated_token::mint = nft_mint, 
        associated_token::authority = payer, 
        constraint = nft_token.amount == 1
    )]
    pub nft_token: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"metadata", Metadata::id().as_ref(), nft_mint.key().as_ref()],
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
    #[account(seeds = [b"nft_authority", sponsor.key().as_ref()], bump = sponsor.auth_rules_bump)]
    pub nft_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer, 
        associated_token::mint = nft_mint, 
        associated_token::authority = nft_authority
    )]
    pub nft_custody: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"metadata", 
            Metadata::id().as_ref(),
            nft_mint.key().as_ref(),
            b"token_record",
            nft_token.key().as_ref(),
        ],
        seeds::program = Metadata::id(),
        bump
    )]
    pub source_token_record: Account<'info, TokenRecordAccount>,
    
    /// CHECK: account constraints checked in account trait
    #[account(
        mut,
        seeds = [b"metadata", 
            Metadata::id().as_ref(),
            nft_mint.key().as_ref(),
            b"token_record",
            nft_custody.key().as_ref(),
        ],
        seeds::program = Metadata::id(),
        bump
    )]
    pub destination_token_record: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,

    // /// CHECK: account constraints checked in account trait
    // #[account(address = mpl_token_auth_rules::id())]
    // pub auth_rules_program: UncheckedAccount<'info>,

    // /// CHECK: account constraints checked in account trait
    // #[account(owner = mpl_token_auth_rules::id())]
    // pub auth_rules: UncheckedAccount<'info>,
}