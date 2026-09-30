use anchor_lang::prelude::*;
use crate::{errors::OathError, state::*};

#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = 8 + ProtocolConfig::INIT_SPACE, seeds = [b"protocol"], bump)]
    pub protocol: Account<'info, ProtocolConfig>,
    #[account(constraint = program.programdata_address()? == Some(program_data.key()) @ OathError::InvalidUpgradeAuthority)]
    pub program: Program<'info, crate::program::OathCore>,
    #[account(constraint = program_data.upgrade_authority_address == Some(authority.key()) @ OathError::InvalidUpgradeAuthority)]
    pub program_data: Account<'info, ProgramData>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeProtocol>, treasury: Pubkey) -> Result<()> {
    require_keys_neq!(treasury, Pubkey::default(), OathError::WrongRecipient);
    ctx.accounts.protocol.set_inner(ProtocolConfig { authority: ctx.accounts.authority.key(), treasury, paused: false });
    Ok(())
}
