//! Launch gate management. Every instruction here requires the `ProtocolConfig.authority`
//! signer and touches only the `LaunchGate` account. None of them can reach a vault, an
//! Oath, a milestone, a reviewer set or a failure destination (docs/THREAT-MODEL.md
//! section 4): the admin can decide who may create, never what happens to what was created.
use anchor_lang::prelude::*;
use crate::{errors::OathError, state::*};

#[derive(Accounts)]
pub struct InitializeLaunchGate<'info> {
    #[account(mut)] pub authority: Signer<'info>,
    #[account(seeds = [b"protocol"], bump, has_one = authority @ OathError::Unauthorized)]
    pub protocol: Account<'info, ProtocolConfig>,
    #[account(init, payer = authority, space = 8 + LaunchGate::INIT_SPACE, seeds = [b"launch_gate"], bump)]
    pub launch_gate: Account<'info, LaunchGate>,
    pub system_program: Program<'info, System>,
}

/// Shared by set_launches_permissionless, add_allowed_creator and remove_allowed_creator.
#[derive(Accounts)]
pub struct ManageLaunchGate<'info> {
    pub authority: Signer<'info>,
    #[account(seeds = [b"protocol"], bump, has_one = authority @ OathError::Unauthorized)]
    pub protocol: Account<'info, ProtocolConfig>,
    #[account(mut, seeds = [b"launch_gate"], bump)]
    pub launch_gate: Account<'info, LaunchGate>,
}

/// Starts closed: nobody may create until the authority allowlists a creator or opens the gate.
pub fn initialize(ctx: Context<InitializeLaunchGate>) -> Result<()> {
    ctx.accounts.launch_gate.set_inner(LaunchGate { launches_permissionless: false, allowed_creators: Vec::new() });
    Ok(())
}

pub fn set_permissionless(ctx: Context<ManageLaunchGate>, permissionless: bool) -> Result<()> {
    ctx.accounts.launch_gate.launches_permissionless = permissionless;
    Ok(())
}

pub fn add_creator(ctx: Context<ManageLaunchGate>, creator: Pubkey) -> Result<()> {
    require_keys_neq!(creator, Pubkey::default(), OathError::InvalidReviewer);
    let gate = &mut ctx.accounts.launch_gate;
    require!(!gate.allowed_creators.contains(&creator), OathError::AlreadyAllowlisted);
    require!(gate.allowed_creators.len() < MAX_ALLOWED_CREATORS, OathError::AllowlistFull);
    gate.allowed_creators.push(creator);
    Ok(())
}

pub fn remove_creator(ctx: Context<ManageLaunchGate>, creator: Pubkey) -> Result<()> {
    let gate = &mut ctx.accounts.launch_gate;
    let position = gate.allowed_creators.iter().position(|k| *k == creator).ok_or(OathError::NotAllowlisted)?;
    gate.allowed_creators.swap_remove(position);
    Ok(())
}
