use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, CloseAccount, Mint, Token, TokenAccount, Transfer},
};
use crate::{constants::NATIVE_MINT_STR, error::MemepodError, MainState, PodState};
use std::str::FromStr;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct WithdrawInput {
    pub base_amount: u64,
    pub quote_amount: u64,
}

pub fn withdraw(ctx: Context<AWithdrawState>, input: WithdrawInput) -> Result<()> {
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

    let admin_base_ata = ctx.accounts.admin_base_ata.to_account_info();
    let admin_quote_ata = ctx.accounts.admin_quote_ata.to_account_info();
    let token_program = ctx.accounts.token_program.to_account_info();

    // send tokens in pool and virt
    if (input.base_amount > 0) {
        require!(
            (pod_state.base_amount - pod_state.bought_amount) > input.base_amount,
            MemepodError::InsufficientFund
        );

        pod_state.base_amount -= input.base_amount;

        let pod_base_transfer_cpi_account = Transfer {
            from: ctx.accounts.reserver_base_ata.to_account_info(),
            to: admin_base_ata.clone(),
            authority: pod_state.to_account_info(),
        };

        token::transfer(
            CpiContext::new_with_signer(
                token_program.clone(),
                pod_base_transfer_cpi_account,
                &[&[
                    PodState::PREFIX_SEED,
                    pod_state.base_mint.as_ref(),
                    pod_state.quote_mint.as_ref(),
                    pod_state.owner.as_ref(),
                    &[ctx.bumps.pod_state],
                ]],
            ),
            input.base_amount,
        )?;
    }

    // send SOL in pool
    if (input.quote_amount > 0) {

        let pod_quote_transfer_cpi_account = Transfer {
            from: ctx.accounts.reserver_quote_ata.to_account_info(),
            to: admin_quote_ata.clone(),
            authority: pod_state.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(
                token_program.clone(),
                pod_quote_transfer_cpi_account,
                &[&[
                    PodState::PREFIX_SEED,
                    pod_state.base_mint.as_ref(),
                    pod_state.quote_mint.as_ref(),
                    pod_state.owner.as_ref(),
                    &[ctx.bumps.pod_state],
                ]],
            ),
            input.quote_amount,
        )?;

        let close_ata_cpi_accounts = CloseAccount {
            account: admin_quote_ata.to_account_info(),
            authority: admin.clone(),
            destination: admin,
        };

        token::close_account(CpiContext::new(token_program, close_ata_cpi_accounts))?;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct AWithdrawState<'info> {
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
    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = quote_mint,
        associated_token::authority = pod_state,
    )]
    pub reserver_quote_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = base_mint,
        associated_token::authority = admin,
    )]
    pub admin_base_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = quote_mint,
        associated_token::authority = admin,
    )]
    pub admin_quote_ata: Box<Account<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
