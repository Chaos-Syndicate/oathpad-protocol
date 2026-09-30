use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::Mint;
use crate::{dbc, errors::OathError, state::*};

#[derive(Accounts)]
pub struct CreateLaunchBinding<'info> {
    #[account(mut)] pub creator: Signer<'info>,
    #[account(seeds = [b"protocol"], bump, constraint = !protocol.paused @ OathError::Paused)]
    pub protocol: Account<'info, ProtocolConfig>,
    /// Fails closed: the gate account must exist (initialize_launch_gate) and allow the creator.
    #[account(seeds = [b"launch_gate"], bump, constraint = launch_gate.allows(&creator.key()) @ OathError::LaunchesGated)]
    pub launch_gate: Account<'info, LaunchGate>,
    pub token_mint: Account<'info, Mint>,
    /// CHECK: two accepted shapes, decided by the account owner in the handler. Owned by the
    /// Meteora DBC program: must be a `VirtualPool` whose creator, base_mint and config match
    /// (proof of launch ownership; the mint authority is irrelevant because DBC revokes it at
    /// pool creation). Any other owner: opaque metadata, and the creator must hold the mint
    /// authority (the V1 path for non-DBC mints).
    pub dbc_pool: UncheckedAccount<'info>,
    #[account(init, payer = creator, space = 8 + LaunchBinding::INIT_SPACE,
        seeds = [b"launch", token_mint.key().as_ref()], bump)]
    pub launch: Account<'info, LaunchBinding>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateLaunchBinding>, dbc_config: Pubkey) -> Result<()> {
    let creator = ctx.accounts.creator.key();
    let token_mint = ctx.accounts.token_mint.key();
    let pool = ctx.accounts.dbc_pool.to_account_info();
    require_keys_neq!(pool.key(), Pubkey::default(), OathError::InvalidPda);
    require_keys_neq!(dbc_config, Pubkey::default(), OathError::InvalidPda);

    if dbc::is_dbc_owned(&pool) {
        // Proof of DBC launch ownership. A DBC pool can never be bound by anyone but its
        // creator, for any mint but its base mint, or under any config but its own.
        let facts = dbc::read_virtual_pool(&pool)?;
        require_keys_eq!(facts.base_mint, token_mint, OathError::PoolMintMismatch);
        require_keys_eq!(facts.creator, creator, OathError::PoolCreatorMismatch);
        require_keys_eq!(facts.config, dbc_config, OathError::PoolConfigMismatch);
    } else {
        // Non-DBC mint: the V1 rule. The pool/config keys are stored as metadata only.
        require!(ctx.accounts.token_mint.mint_authority == COption::Some(creator), OathError::Unauthorized);
    }

    ctx.accounts.launch.set_inner(LaunchBinding {
        creator, token_mint, dbc_pool: pool.key(), dbc_config, oath_count: 0, status: LaunchStatus::Draft,
    });
    Ok(())
}
