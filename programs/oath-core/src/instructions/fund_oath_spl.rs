use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};
use crate::{errors::OathError, state::*};

#[derive(Accounts)]
pub struct FundOathSpl<'info> {
    #[account(mut)] pub funder: Signer<'info>,
    #[account(constraint = super::valid_oath(&oath) @ OathError::InvalidPda)]
    pub oath: Account<'info, Oath>,
    #[account(address = oath.asset_mint @ OathError::WrongMint)] pub mint: Account<'info, Mint>,
    #[account(mut, token::mint = mint, token::authority = funder)] pub source: Account<'info, TokenAccount>,
    #[account(init_if_needed, payer = funder, seeds = [b"vault", oath.key().as_ref()], bump,
        token::mint = mint, token::authority = vault, address = oath.vault @ OathError::WrongVault)]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<FundOathSpl>, amount: u64) -> Result<()> {
    require!(ctx.accounts.oath.status == OathStatus::Draft, OathError::MutationAfterActivation);
    require!(amount > 0, OathError::ZeroAmount);
    super::safe_token_vault(&ctx.accounts.vault, ctx.accounts.vault.key(), ctx.accounts.mint.key())?;
    // `amount` is the maximum contribution, not an exact transfer request.
    let amount = super::funding_amount(ctx.accounts.vault.amount, ctx.accounts.oath.committed_amount, amount)?;
    if amount == 0 {
        return Ok(());
    }
    token::transfer_checked(CpiContext::new(ctx.accounts.token_program.to_account_info(), TransferChecked {
        from: ctx.accounts.source.to_account_info(), mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.vault.to_account_info(), authority: ctx.accounts.funder.to_account_info(),
    }), amount, ctx.accounts.mint.decimals)
}
