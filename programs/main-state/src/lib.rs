#![allow(unused)]

use anchor_lang::prelude::*;

pub mod main_state;
pub mod pod;

pub mod constants;
pub mod error;
pub mod utils;

use main_state::*;
use pod::*;

declare_id!("5EFN2ja837Uk3setSnu99JvSfx8H8sNWKx3Hndm3XeKb");

#[program]
pub mod memepod {
    use super::*;

    pub fn init_main_state(ctx: Context<AInitMainState>) -> Result<()> {
        main_state::init_main_state(ctx)
    }

    pub fn update_main_state(ctx: Context<AUpdateMainState>, input: UpdateMainStateInput) -> Result<()> {
        main_state::update_main_state(ctx, input)
    }
    
    pub fn create_pod(ctx: Context<ACreatePod>, input: CreatePodInput) -> Result<()> {
        pod::create_pod(ctx, input)
    }

    pub fn buy(ctx: Context<ABuy>, amount: u64) -> Result<()> {
        pod::buy(ctx, amount)
    }
    
    pub fn withdraw(ctx: Context<AWithdrawState>, input: WithdrawInput) -> Result<()> {
        pod::withdraw(ctx, input)
    }

    pub fn close_pod(ctx: Context<AClosePodState>) -> Result<()> {
        pod::close_pod(ctx)
    }

    pub fn edit_pod(ctx: Context<AEditPodState>, input: EditPodInput) -> Result<()> {
        pod::edit_pod(ctx, input)
    }
}
