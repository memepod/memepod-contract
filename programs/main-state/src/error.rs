use anchor_lang::prelude::error_code;

#[error_code]
pub enum MemepodError {
    #[msg("Uninitialized")]
    Uninitialized,

    #[msg("AlreadyInitialized")]
    AlreadyInitialized,

    #[msg("Unauthorised")]
    Unauthorised,

    #[msg("Insufficient fund")]
    InsufficientFund,

    #[msg("One token should be Sol")]
    UnknownToken,

    #[msg("Not Actived")]
    NotActive,

    #[msg("Pod name too long")]
    PodNameTooLong,

    #[msg("Token name too long")]
    TokenNameTooLong,
    
    #[msg("Token symbol too long")]
    TokenSymbolTooLong,
}
