use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_lang::solana_program::sysvar;
use anchor_spl::{
    associated_token::AssociatedToken, metadata::{mpl_token_metadata::instructions::TransferV1CpiBuilder, MasterEditionAccount, Metadata, MetadataAccount, TokenRecordAccount}, token::{self, Mint, Token, TokenAccount}
};

use crate::util::FEE_WALLETS;

use crate::Sponsor;

use crate::error::HybridErrorCode;

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

        const PREFIX_SEED: &'static [u8] = b"hybrid_defi";
        let signer_seeds = [PREFIX_SEED, sponsor.authority.as_ref(), sponsor.nft_mint.as_ref(), &[sponsor.auth_rules_bump]];

        let mut symbol_iter = ctx.accounts.nft_metadata.symbol.chars();

        let factor = sponsor.swap_factor[0] as f64;

        require!(
            sponsor.swap_factor[0] * sponsor.swap_factor[2] > ctx.accounts.sponsor_token_account.amount as f64,
            HybridErrorCode::UnbalancedPool
        );

        match symbol_iter.nth(2) {
            Some('C') => token::transfer(
                        CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(),
                            token::Transfer {
                                from: ctx.accounts.sponsor_token_account.to_account_info(),
                                to: ctx.accounts.payer_token_account.to_account_info(),
                                authority: sponsor.to_account_info(),
                            },
                            &[&signer_seeds]
                        ),
                        sponsor.swap_factor[0] as u64,
                    )?,
            Some('R') => token::transfer(
                        CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(),
                            token::Transfer {
                                from: ctx.accounts.sponsor_token_account.to_account_info(),
                                to: ctx.accounts.payer_token_account.to_account_info(),
                                authority: sponsor.to_account_info(),
                            },
                            &[&signer_seeds]
                        ),
                        (factor * sponsor.swap_factor[1] as f64) as u64,
                    )?,
            Some('L') => token::transfer(
                        CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(),
                            token::Transfer {
                                from: ctx.accounts.sponsor_token_account.to_account_info(),
                                to: ctx.accounts.payer_token_account.to_account_info(),
                                authority: sponsor.to_account_info(),
                            },
                            &[&signer_seeds]
                        ),
                        (factor * sponsor.swap_factor[2] as f64) as u64,
                    )?,
            None => panic!("Symbol did not match any defined schema."),
            _ => panic!("An unexpected error occurred.")
        };

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.fee_wallet.to_account_info(),
                },
            ),
            150000,
        )?;  

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.fee_wallet_two.to_account_info(),
                },
            ),
            150000,
        )?;  

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.fee_wallet_three.to_account_info(),
                },
            ),
            300000,
        )?;  

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

    #[account(mut)]
    pub token_mint: Box<Account<'info, token::Mint>>,
    #[account(
        mut,
        token::mint = sponsor.token_mint,
        token::authority = sponsor,
    )]
    pub sponsor_token_account: Box<Account<'info, token::TokenAccount>>,
    #[account(
        mut,
        token::mint = sponsor.token_mint,
        token::authority = payer,
    )]
    pub payer_token_account: Account<'info, token::TokenAccount>,

    #[account(mint::decimals = 0, constraint = nft_mint.supply == 1)]
    pub nft_mint: Box<Account<'info, Mint>>,

    #[account(
        mut, 
        associated_token::mint = nft_mint, 
        associated_token::authority = payer, 
        constraint = nft_token.amount == 1
    )]
    pub nft_token: Box<Account<'info, TokenAccount>>,
    
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
    pub nft_custody: Box<Account<'info, TokenAccount>>,

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
    pub source_token_record: Box<Account<'info, TokenRecordAccount>>,
    
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

    #[account(
        mut,
        constraint = fee_wallet.key().to_string().as_str() == FEE_WALLETS[0]
    )]
    /// CHECK: This isn't unsafe because I said so
    pub fee_wallet: AccountInfo<'info>,

    #[account(
        mut,
        constraint = fee_wallet_two.key().to_string().as_str() == FEE_WALLETS[1]
    )]
    /// CHECK: This isn't unsafe because I said so
    pub fee_wallet_two: AccountInfo<'info>,

    #[account(
        mut,
        constraint = fee_wallet_three.key().to_string().as_str() == FEE_WALLETS[2]
    )]
    /// CHECK: This isn't unsafe because I said so
    pub fee_wallet_three: AccountInfo<'info>,

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
