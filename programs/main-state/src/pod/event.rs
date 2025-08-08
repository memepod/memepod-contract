use anchor_lang::prelude::*;

#[event]
pub struct CreateEvent {
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub base_amount: u64,
    pub token_price: u64,
    pub expire_time: u64,
    pub timestamp: i64,
}

#[event]
pub struct BuyEvent {
    pub user: Pubkey,
    pub base_mint: Pubkey,
    pub quote_amount: u64,
    pub base_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct CompleteEvent {
    pub user: Pubkey,
    pub base_mint: Pubkey,
    pub timestamp: i64,
}
