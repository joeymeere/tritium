use anchor_lang::prelude::*;

#[error_code]
pub enum HybridErrorCode {
    #[msg("Pool initializer is not apart of the whitelist")]
    UnauthorizedCreation, 

    #[msg("There are insufficient tokens to satisfy this swap.")]
    UnbalancedPool,

    #[msg("Symbol schema does not match the expected schema.")]
    InvalidSymbol,

    #[msg("Insufficent tokens provided for swap.")]
    InsufficientTokens,

    #[msg("bad metadata passed")]
    BadMetadata,
}