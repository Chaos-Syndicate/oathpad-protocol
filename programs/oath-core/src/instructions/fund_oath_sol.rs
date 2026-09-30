use anchor_lang::{prelude::*, system_program::{self, Transfer}};
use crate::{errors::OathError, state::*};

#[derive(Accounts)]
pub struct FundOathSol<'info> {
    #[account(mut)] pub funder: Signer<'info>,
    #[account(constraint = super::valid_oath(&oath) @ OathError::InvalidPda)]
    pub oath: Account<'info, Oath>,
    /// CHECK: system-owned, zero-data PDA; only this program can sign transfers.
    #[account(mut, seeds = [b"vault", oath.key().as_ref()], bump,
        address = oath.vault @ OathError::WrongVault, owner = system_program::ID)]
    pub vault: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<FundOathSol>, amount: u64) -> Result<()> {
    require!(ctx.accounts.oath.status == OathStatus::Draft, OathError::MutationAfterActivation);
    require_keys_eq!(ctx.accounts.oath.asset_mint, Pubkey::default(), OathError::WrongAssetType);
    require!(ctx.accounts.vault.data_is_empty(), OathError::WrongVault);
    require!(amount > 0, OathError::ZeroAmount);
    // `amount` is the maximum contribution. Include any still-missing rent reserve
    // in the live shortfall so a prior donation only reduces the transfer.
    let cap = ctx.accounts.oath.committed_amount.checked_add(Rent::get()?.minimum_balance(0)).ok_or(OathError::ArithmeticOverflow)?;
    let amount = super::funding_amount(ctx.accounts.vault.lamports(), cap, amount)?;
    if amount == 0 {
        return Ok(());
    }
    system_program::transfer(CpiContext::new(ctx.accounts.system_program.to_account_info(), Transfer {
        from: ctx.accounts.funder.to_account_info(), to: ctx.accounts.vault.to_account_info(),
    }), amount)
}
