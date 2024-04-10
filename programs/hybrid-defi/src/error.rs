use anchor_lang::prelude::*;

#[error_code]
pub enum HybridErrorCode {
    #[msg("Insufficent tokens provided for swap.")]
    InsufficientTokens,

    #[msg("bad metadata passed")]
    BadMetadata,
}