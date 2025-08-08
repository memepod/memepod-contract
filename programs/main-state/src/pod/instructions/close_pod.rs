use anchor_lang::prelude::*;
use anchor_spl::token::burn;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, CloseAccount, Mint, Token, TokenAccount, Transfer, Burn},
};
use crate::{error::MemepodError, MainState, PodState};
use std::str::FromStr;

pub fn close_pod(ctx: Context<AClosePodState>) -> Result<()> {
    let admin = ctx.accounts.admin.to_account_info();

    let main_state = &ctx.accounts.main_state;
    let token_program = ctx.accounts.token_program.to_account_info();
    require!(
        main_state.initialized.eq(&true),
        MemepodError::Uninitialized
    );
    let pod_state = &mut ctx.accounts.pod_state;

    require!(
        ctx.accounts.admin.key().eq(&pod_state.owner),
        MemepodError::Unauthorised
    );

    require!(pod_state.is_active.eq(&true), MemepodError::NotActive);

    pod_state.is_active = false;

    let cpi_accounts = Burn {
        mint: ctx.accounts.base_mint.to_account_info(),
        from: ctx.accounts.reserver_base_ata.to_account_info(),
        authority: pod_state.to_account_info(),
    };

    let amount = pod_state.base_amount - pod_state.bought_amount; 

    burn(CpiContext::new_with_signer(
                token_program.clone(),
                cpi_accounts,
                &[&[
                    PodState::PREFIX_SEED,
                    pod_state.base_mint.as_ref(),
                    pod_state.quote_mint.as_ref(),
                    pod_state.owner.as_ref(),
                    &[ctx.bumps.pod_state],
                ][..]],
            ), amount)?;

    Ok(())
}

#[derive(Accounts)]
pub struct AClosePodState<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [MainState::PREFIX_SEED],
        bump,
    )]
    pub main_state: Box<Account<'info, MainState>>,

    #[account(
        mut,
        seeds = [
            PodState::PREFIX_SEED,
            base_mint.key().as_ref(), 
            quote_mint.key().as_ref(),
            admin.key().as_ref(),
        ],
        bump,
    )]
    pub pod_state: Box<Account<'info, PodState>>,

    #[account(mut,)]
    pub base_mint: Box<Account<'info, Mint>>,
    
    #[account(mut,)]
    pub quote_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = base_mint,
        associated_token::authority = pod_state,
    )]
    pub reserver_base_ata: Box<Account<'info, TokenAccount>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

}
