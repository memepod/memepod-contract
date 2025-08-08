use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, CloseAccount, Mint, Token, TokenAccount, Transfer},
};
use crate::{error::MemepodError, MainState, PodState, utils::{check_balance_on_pod_creator}};
use std::str::FromStr;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct EditPodInput {
    pub token_price: u64,
    pub base_amount: u64
}

pub fn edit_pod(ctx: Context<AEditPodState>, input: EditPodInput) -> Result<()> {
    let admin = ctx.accounts.admin.to_account_info();

    let main_state = &ctx.accounts.main_state;
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

    let token_program = ctx.accounts.token_program.to_account_info();

    //transfer
    let base_transfer_cpi_accounts = Transfer {
        from: ctx.accounts.creator_base_ata.to_account_info(),
        to: ctx.accounts.reserver_base_ata.to_account_info(),
        authority: admin.clone(),
    };

    token::transfer(
        CpiContext::new(token_program.to_account_info(), base_transfer_cpi_accounts),
        input.base_amount,
    )?;

    pod_state.token_price = input.token_price;
    pod_state.base_amount += input.base_amount;

    Ok(())
}

#[derive(Accounts)]
#[instruction(input: EditPodInput)]
pub struct AEditPodState<'info> {
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
        payer=admin,
        associated_token::mint =base_mint,
        associated_token::authority = admin,
        constraint = check_balance_on_pod_creator(creator_base_ata.as_ref(), input.base_amount) @ MemepodError::InsufficientFund
    )]
    pub creator_base_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer=admin,
        associated_token::mint = base_mint,
        associated_token::authority = pod_state,
    )]
    pub reserver_base_ata: Box<Account<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
