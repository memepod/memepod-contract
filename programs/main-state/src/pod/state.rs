use anchor_lang::prelude::*;

#[account]
pub struct PodState {
    pub pod_name: [u8; 32],
    pub token_name: [u8; 32],
    pub token_symbol: [u8; 10],
    pub decimal: u8,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub owner: Pubkey,
    pub base_amount: u64,
    pub bought_amount: u64,
    pub token_price: u64, // lamports
    pub expire_time: u64,
    pub is_active: bool
}

impl PodState {
    pub const MAX_SIZE: usize = std::mem::size_of::<Self>();
    pub const PREFIX_SEED: &'static [u8] = b"memepod";

    pub fn compute_receivable_amount_on_buy(&mut self, quote_amount: u64) -> u64 {
        let base_amount = (quote_amount as u128)
            .checked_mul(self.token_price as u128)
            .unwrap()
            .checked_div(1000000000)
            .unwrap();
        self.bought_amount += base_amount as u64;
        base_amount as u64
    }
}
