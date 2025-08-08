use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, SyncNative, Token, TokenAccount, Transfer},
};
use crate::{
    constants::NATIVE_MINT_STR,
    error::MemepodError,
    utils::{check_balance_on_pod_creator, sync_native_amount},
    CreateEvent, MainState, PodState,
};

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct CreatePodInput {
    pub pod_name: String,
    pub base_amount: u64,
    pub token_price: u64,
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimal: u8,
    pub expire_time: u64,
}

const MAX_POD_NAME_LEN: usize = 32;
const MAX_TOKEN_NAME_LEN: usize = 32;
const MAX_TOKEN_SYMBOL_LEN: usize = 10;

fn str_to_fixed_bytes<const N: usize>(s: &str) -> [u8; N] {
    let mut buf = [0u8; N];
    let bytes = s.as_bytes();
    let len = bytes.len().min(N);
    buf[..len].copy_from_slice(&bytes[..len]);
    buf
}

pub fn create_pod(ctx: Context<ACreatePod>, input: CreatePodInput) -> Result<()> {
    let main_state = &mut ctx.accounts.main_state;
    require!(
        main_state.initialized.eq(&true),
        MemepodError::Uninitialized
    );

    require!(input.pod_name.len() <= MAX_POD_NAME_LEN, MemepodError::PodNameTooLong);
    require!(input.token_name.len() <= MAX_TOKEN_NAME_LEN, MemepodError::TokenNameTooLong);
    require!(input.token_symbol.len() <= MAX_TOKEN_SYMBOL_LEN, MemepodError::TokenSymbolTooLong);

    let pod_state = &mut ctx.accounts.pod_state;
    let creator = ctx.accounts.creator.to_account_info();
    let system_program = ctx.accounts.system_program.to_account_info();
    let token_program = ctx.accounts.token_program.to_account_info();
    let creator_base_ata = &ctx.accounts.creator_base_ata;
    let creator_quote_ata = &ctx.accounts.creator_quote_ata;

    pod_state.owner = creator.key();
    pod_state.base_mint = creator_base_ata.mint;
    pod_state.quote_mint = creator_quote_ata.mint;
    pod_state.base_amount = input.base_amount;
    pod_state.pod_name = str_to_fixed_bytes(&input.pod_name);
    pod_state.token_name = str_to_fixed_bytes(&input.token_name);
    pod_state.token_symbol = str_to_fixed_bytes(&input.token_symbol);
    pod_state.token_price = input.token_price;
    pod_state.expire_time = input.expire_time;
    pod_state.decimal = input.token_decimal;
    pod_state.is_active = true;

    //handler wrap sol
    if (creator_quote_ata.mint.to_string() == NATIVE_MINT_STR) {
        sync_native_amount(
            creator.clone(),
            creator_quote_ata,
            main_state.creation_fee,
            system_program.clone(),
            token_program.clone(),
        )?;
    }

    //transfer
    let base_transfer_cpi_accounts = Transfer {
        from: ctx.accounts.creator_base_ata.to_account_info(),
        to: ctx.accounts.reserver_base_ata.to_account_info(),
        authority: creator.clone(),
    };

    token::transfer(
        CpiContext::new(token_program.to_account_info(), base_transfer_cpi_accounts),
        input.base_amount,
    )?;
    
    let quote_transfer_cpi_accounts = Transfer {
        from: ctx.accounts.creator_quote_ata.to_account_info(),
        to: ctx.accounts.fee_quote_ata.to_account_info(),
        authority: creator.clone(),
    };
    token::transfer(
        CpiContext::new(token_program.to_account_info(), quote_transfer_cpi_accounts),
        main_state.creation_fee,
    )?;

    emit!(CreateEvent {
        creator: pod_state.owner,
        base_mint: pod_state.base_mint,
        base_amount: pod_state.base_amount,
        token_price: pod_state.token_price,
        expire_time: pod_state.expire_time,
        timestamp: Clock::get()?.unix_timestamp
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(input: CreatePodInput)]
pub struct ACreatePod<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [MainState::PREFIX_SEED],
        bump,
    )]
    pub main_state: Box<Account<'info, MainState>>,
    #[account(
        init,
        payer = creator,
        seeds =[
            PodState::PREFIX_SEED,
            base_mint.key().as_ref(),
            quote_mint.key().as_ref(),
            creator.key().as_ref()
        ],
        bump,
        space = 16 + PodState::MAX_SIZE
    )]
    pub pod_state: Box<Account<'info, PodState>>,

    pub base_mint: Box<Account<'info, Mint>>,
    #[account(constraint = quote_mint.key().to_string() == NATIVE_MINT_STR @ MemepodError::UnknownToken)]
    pub quote_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer=creator,
        associated_token::mint =base_mint,
        associated_token::authority = creator,
        constraint = check_balance_on_pod_creator(creator_base_ata.as_ref(), input.base_amount) @ MemepodError::InsufficientFund
    )]
    pub creator_base_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer=creator,
        associated_token::mint =quote_mint,
        associated_token::authority = creator,
        constraint = check_balance_on_pod_creator(creator_quote_ata.as_ref(), main_state.creation_fee) @ MemepodError::InsufficientFund
    )]
    pub creator_quote_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut, address = main_state.fee_recipient,)]
    /// CHECK: this should be set by admin
    pub fee_recipient: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = quote_mint,
        associated_token::authority = fee_recipient,
    )]
    /// CHECK: this should be set by fee_recipient
    pub fee_quote_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer=creator,
        associated_token::mint = base_mint,
        associated_token::authority = pod_state,
    )]
    pub reserver_base_ata: Box<Account<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
