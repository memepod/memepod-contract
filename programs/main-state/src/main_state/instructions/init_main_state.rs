use crate::{
    MainState,
    error::MemepodError
};
use anchor_lang::prelude::*;

pub fn init_main_state(ctx: Context<AInitMainState>) -> Result<()> {
    let state = &mut ctx.accounts.main_state;
    require!(state.initialized.eq(&false), MemepodError::AlreadyInitialized);

    state.initialized = true;
    state.owner = ctx.accounts.owner.key();
    state.fee_recipient = ctx.accounts.owner.key();
    state.trading_fee = 1_000;   // default: 1%
    state.creator_fee = 1_000;
    state.owner_fee = 1_000;
    state.creation_fee = 100000000; // 0.1 SOL
    Ok(())
}

#[derive(Accounts)]
pub struct AInitMainState<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = owner,
        seeds = [MainState::PREFIX_SEED],
        bump,
        space = 8 + MainState::MAX_SIZE
    )]
    pub main_state: Account<'info, MainState>,

    pub system_program: Program<'info, System>,
}
