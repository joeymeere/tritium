use anchor_lang::prelude::*;

#[error_code]
pub enum HybridErrorCode {
    #[msg("bad metadata passed")]
    BadMetadata,
}