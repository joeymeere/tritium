pub const FEE_WALLETS: [&'static str; 3] = ["EYNsuoUh4pRCpuNqj5cH8zUDXST4o8YYqRg6vraG7Br7", "5ZnZRLxJZo6MiUBvfNQaB6aFSjNx5sv3Zib2YHHEeGQv", "ghosnnrbJRNUueziNL579JZCqvcLpdHSMXU2zn9uGJS"];

/* 
use std::collections::HashMap;

use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};
use mpl_token_auth_rules::payload::{Payload, PayloadType, ProofInfo, SeedsVec};
use mpl_token_metadata::instruction::builders::TransferBuilder;
use mpl_token_metadata::instruction::TransferArgs;
use mpl_token_metadata::processor::AuthorizationData;
use mpl_token_metadata::state::{Metadata, TokenMetadataAccount, TokenStandard, ProgrammableConfig::V1};

use crate::*;

use crate::error::HybridErrorCode;

pub fn sent_pnft<'info>(
    //these 3 can be the same, but not necessarily
    authority: &AccountInfo<'info>,
    owner: &AccountInfo<'info>,
    //(!) payer can't carry data, has to be a normal KP:
    // https://github.com/solana-labs/solana/blob/bda0c606a19ce1cc44b5ab638ff0b993f612e76c/runtime/src/system_instruction_processor.rs#L197
    payer: &AccountInfo<'info>,
    source_ata: &Account<'info, TokenAccount>,
    dest_ata: &Account<'info, TokenAccount>,
    dest_owner: &AccountInfo<'info>,
    nft_mint: &Account<'info, Mint>,
    nft_metadata: &UncheckedAccount<'info>,
    nft_edition: &UncheckedAccount<'info>,
    system_program: &Program<'info, System>,
    token_program: &Program<'info, Token>,
    ata_program: &Program<'info, AssociatedToken>,
    metadata_program: &UncheckedAccount<'info>,
    instructions: &UncheckedAccount<'info>,
    owner_token_record: &UncheckedAccount<'info>,
    dest_token_record: &UncheckedAccount<'info>,
    authorization_rules_program: &UncheckedAccount<'info>,
    rules_acc: Option<&AccountInfo<'info>>,
    authorization_data: Option<AuthorizationDataLocal>,
    //if passed, use signed_invoke() instead of invoke()
    program_signer: Option<&AccountInfo<'info>>,
) -> Result<()> {
    let mut builder = TransferBuilder::new();
    
    builder
        .authority(*authority.key)
        .token_owner(*owner.key)
        .token(source_ata.key())
        .destination_owner(*dest_owner.key)
        .destination_owner(dest_ata.key())
        .mint(nft_mint.key())
        .metadata(nft_metadata.key())
        .edition(nft_edition.key())
        .payer(*payer.key);

    let mut account_infos = vec![
        //   0. `[writable]` Token account
        source_ata.to_account_info(),
        //   1. `[]` Token account owner
        owner.to_account_info(),
        //   2. `[writable]` Destination token account
        dest_ata.to_account_info(),
        //   3. `[]` Destination token account owner
        dest_owner.to_account_info(),
        //   4. `[]` Mint of token asset
        nft_mint.to_account_info(),
        //   5. `[writable]` Metadata account
        nft_metadata.to_account_info(),
        //   6. `[optional]` Edition of token asset
        nft_edition.to_account_info(),
        //   7. `[signer] Transfer authority (token or delegate owner)
        authority.to_account_info(),
        //   8. `[optional, writable]` Owner record PDA
        //passed in below, if needed
        //   9. `[optional, writable]` Destination record PDA
        //passed in below, if needed
        //   10. `[signer, writable]` Payer
        payer.to_account_info(),
        //   11. `[]` System Program
        system_program.to_account_info(),
        //   12. `[]` Instructions sysvar account
        instructions.to_account_info(),
        //   13. `[]` SPL Token Program
        token_program.to_account_info(),
        //   14. `[]` SPL Associated Token Account program
        ata_program.to_account_info(),
        //   15. `[optional]` Token Authorization Rules Program
        //passed in below, if needed
        //   16. `[optional]` Token Authorization Rules account
        //passed in below, if needed
    ];

    let metadata = assert_decode_metadata(nft_mint, &nft_metadata.to_account_info())?;
    if let Some(standard) = metadata.token_standard {
        msg!("standard triggered");
        if standard == TokenStandard::ProgrammableNonFungible {
            //1. add to builder
            builder
                .owner_token_record(owner_token_record.key())
                .destination_token_record(dest_token_record.key());

            //2. add to accounts (if try to pass these for non-pNFT, will get owner errors, since they don't exist)
            account_infos.push(owner_token_record.to_account_info());
            account_infos.push(dest_token_record.to_account_info());
        }
    }

    //if auth rules passed in, validate & include it in CPI call
    if let Some(config) = metadata.programmable_config {
        match config {
            V1 { rule_set } => {
                if let Some(rule_set) = rule_set {
                    msg!("ruleset triggered");
                    //safe to unwrap here, it's expected
                    let rules_acc = rules_acc.unwrap();

                    //1. validate
                    if rule_set != *rules_acc.key {
                        // change this
                        
                    }

                    //2. add to builder
                    builder.authorization_rules(*rules_acc.key);
                    builder.authorization_rules_program(*authorization_rules_program.key);

                    //3. add to accounts
                    account_infos.push(authorization_rules_program.to_account_info());
                    account_infos.push(rules_acc.to_account_info());
                }
            }
        }
    }

    let mut transfer_ix = builder
        .build(TransferArgs::V1 {
            amount: 1, //currently 1 only
            authorization_data: authorization_data
                .map(|authorization_data| AuthorizationData::try_from(authorization_data).unwrap()),
            })
        .unwrap();

        transfer_ix = Instruction::from(transfer_ix.try_into().unwrap());

        // if there's a vault, we need to invoke_signed
        invoke(&*transfer_ix, &account_infos)?;

    Ok(())
}

#[inline(never)]
    pub fn assert_decode_metadata<'info>(
        nft_mint: &Account<'info, Mint>,
        metadata_account: &AccountInfo<'info>,
    ) -> Result<Metadata> {
        let (key, _) = Pubkey::find_program_address(
            &[
                mpl_token_metadata::state::PREFIX.as_bytes(),
                mpl_token_metadata::id().as_ref(),
                nft_mint.key().as_ref(),
            ],
            &mpl_token_metadata::id(),
        );
        if key != *metadata_account.key {
            return Err(error!(HybridErrorCode::BadMetadata));
        }
        // Check account owner (redundant because of find_program_address above, but why not).
        if *metadata_account.owner != mpl_token_metadata::id() {
            return Err(error!(HybridErrorCode::BadMetadata));
        }

        Ok(Metadata::from_account_info(metadata_account)?)
    }

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct AuthorizationDataLocal {
    pub payload: Vec<TaggedPayload>,
}
impl Into<AuthorizationData> for AuthorizationDataLocal {
    fn into(self) -> AuthorizationData {
        let mut p = Payload::new();

        self.payload.into_iter().for_each(|tp| {
            p.insert(tp.name, PayloadType::try_from(tp.payload).unwrap());
        });
        AuthorizationData {
            payload: Payload::try_from(p).unwrap(),
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct TaggedPayload {
    name: String,
    payload: PayloadTypeLocal,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum PayloadTypeLocal {
    /// A plain `Pubkey`.
    Pubkey(Pubkey),
    /// PDA derivation seeds.
    Seeds(SeedsVecLocal),
    /// A merkle proof.
    MerkleProof(ProofInfoLocal),
    /// A plain `u64` used for `Amount`.
    Number(u64),
}
impl Into<PayloadType> for PayloadTypeLocal {
    fn into(self) -> PayloadType {
        match self {
            Self::Pubkey(pubkey) => PayloadType::Pubkey(pubkey),
            Self::Seeds(seeds) => PayloadType::Seeds(SeedsVec::try_from(seeds).unwrap()),
            Self::MerkleProof(proof) => {
                PayloadType::MerkleProof(ProofInfo::try_from(proof).unwrap())
            }
            Self::Number(number) => PayloadType::Number(number),
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct SeedsVecLocal {
    /// The vector of derivation seeds.
    pub seeds: Vec<Vec<u8>>,
}
impl Into<SeedsVec> for SeedsVecLocal {
    fn into(self) -> SeedsVec {
        SeedsVec { seeds: self.seeds }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ProofInfoLocal {
    /// The merkle proof.
    pub proof: Vec<[u8; 32]>,
}
impl Into<ProofInfo> for ProofInfoLocal {
    fn into(self) -> ProofInfo {
        ProofInfo { proof: self.proof }
    }
}
*/
