use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use crate::{
    error::MemepodError,
    main_state,
    utils::{calculate_trading_fee, close_token_account, sync_native_amount},
    BuyEvent, CompleteEvent, MainState, PodState,
};

pub fn buy(ctx: Context<ABuy>, amount: u64) -> Result<()> {
    let main_state = &mut ctx.accounts.main_state;
    require!(
        main_state.initialized.eq(&true),
        MemepodError::Uninitialized
    );

    let pod_state = &mut ctx.accounts.pod_state;
    require!(pod_state.is_active.eq(&true), MemepodError::NotActive);

    let buyer = ctx.accounts.buyer.to_account_info();
    let buyer_base_ata = &ctx.accounts.buyer_base_ata;
    let buyer_quote_ata = &ctx.accounts.buyer_quote_ata;
    let token_program = ctx.accounts.token_program.to_account_info();
    let system_program = ctx.accounts.system_program.to_account_info();

    sync_native_amount(
        buyer.clone(),
        &buyer_quote_ata,
        amount,
        system_program.clone(),
        token_program.clone(),
    )?;

    let fee = calculate_trading_fee(main_state.trading_fee, amount);
    let input_amount = amount - fee;
    let output_amount = pod_state.compute_receivable_amount_on_buy(input_amount);

    // sending fee
    let fee_transfer_cpi_account = Transfer {
        from: buyer_quote_ata.to_account_info(),
        to: ctx.accounts.fee_quote_ata.to_account_info(),
        authority: buyer.clone(),
    };
    token::transfer(
        CpiContext::new(token_program.clone(), fee_transfer_cpi_account),
        fee / 2,
    )?;

    // sending input amount (sol)
    let input_amount_transfer_cpi_account = Transfer {
        from: buyer_quote_ata.to_account_info(),
        to: ctx.accounts.reserver_quote_ata.to_account_info(),
        authority: buyer.clone(),
    };
    token::transfer(
        CpiContext::new(token_program.clone(), input_amount_transfer_cpi_account),
        input_amount + fee / 2,
    )?;

    // sending tokens from reserve ata (meme)
    let output_amount_transfer_cpi_account = Transfer {
        from: ctx.accounts.reserver_base_ata.to_account_info(),
        to: buyer_base_ata.to_account_info(),
        authority: pod_state.to_account_info(),
    };
    token::transfer(
        CpiContext::new_with_signer(
            token_program.clone(),
            output_amount_transfer_cpi_account,
            &[&[
                PodState::PREFIX_SEED,
                pod_state.base_mint.as_ref(),
                pod_state.quote_mint.as_ref(),
                pod_state.owner.as_ref(),
                &[ctx.bumps.pod_state],
            ]],
        ),
        output_amount,
    )?;

    // unwrap sol (or closing token account)
    close_token_account(
        buyer.clone(),
        buyer_quote_ata.to_account_info(),
        token_program,
    )?;

    emit!(BuyEvent {
        user: buyer.key(),
        base_mint: pod_state.base_mint,
        base_amount: output_amount,
        quote_amount: amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct ABuy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub creator: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [MainState::PREFIX_SEED],
        bump,
    )]
    pub main_state: Box<Account<'info, MainState>>,

    #[account(mut, address = main_state.fee_recipient,)]
    /// CHECK: this should be set by admin
    pub fee_recipient: AccountInfo<'info>,
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = quote_mint,
        associated_token::authority = fee_recipient,
    )]
    /// CHECK: this should be set by fee_recipient
    pub fee_quote_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [
            PodState::PREFIX_SEED,
            base_mint.key().as_ref(), 
            quote_mint.key().as_ref(),
            creator.key().as_ref()
        ],
        bump,
    )]
    pub pod_state: Box<Account<'info, PodState>>,

    #[account(address = pod_state.base_mint)]
    pub base_mint: Box<Account<'info, Mint>>,
    #[account(address = pod_state.quote_mint)]
    pub quote_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = base_mint,
        associated_token::authority = buyer,
    )]
    pub buyer_base_ata: Box<Account<'info, TokenAccount>>,
    
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = quote_mint,
        associated_token::authority = buyer,
    )]
    pub buyer_quote_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = base_mint,
        associated_token::authority = pod_state,
    )]
    pub reserver_base_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = quote_mint,
        associated_token::authority = pod_state,
    )]
    pub reserver_quote_ata: Box<Account<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
