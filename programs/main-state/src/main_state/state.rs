use anchor_lang::prelude::*;

#[account]
pub struct MainState {
    pub initialized: bool,
    pub owner: Pubkey,
    pub fee_recipient: Pubkey,
    pub creation_fee: u64,
    pub trading_fee: u16,
    pub creator_fee: u16,
    pub owner_fee: u16
}

impl MainState {
    pub const MAX_SIZE: usize = std::mem::size_of::<Self>();
    pub const PREFIX_SEED: &'static [u8] = b"main";
}
