use crate::{error::MemepodError, MainState};
use anchor_lang::prelude::*;

#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone, Copy)]
pub struct UpdateMainStateInput {
    owner: Pubkey,
    fee_recipient: Pubkey,
    creation_fee: u64,
    trading_fee: u16,
    creator_fee: u16,
    owner_fee: u16
}

pub fn update_main_state(
    ctx: Context<AUpdateMainState>,
    input: UpdateMainStateInput,
) -> Result<()> {
    let state = &mut ctx.accounts.main_state;
    require!(state.initialized.eq(&true), MemepodError::Uninitialized);

    state.owner = input.owner;
    state.fee_recipient = input.fee_recipient;
    state.trading_fee = input.trading_fee;
    state.creator_fee = input.creator_fee;
    state.owner_fee = input.owner_fee;
    state.creation_fee = input.creation_fee;
    
    Ok(())
}

#[derive(Accounts)]
pub struct AUpdateMainState<'info> {
    #[account(mut, address = main_state.owner @ MemepodError::Unauthorised)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [MainState::PREFIX_SEED],
        bump,
        has_one = owner,
    )]
    pub main_state: Account<'info, MainState>,
}
