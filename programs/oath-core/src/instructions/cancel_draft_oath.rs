use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};
use crate::{errors::OathError, state::*};

/// Creator self-service recovery, independent of deadlines, configuration, pause
/// and the launch gate. Contributions are not tracked: all assets go to creator.
#[derive(Accounts)]
pub struct CancelDraftOath<'info> {
    pub creator: Signer<'info>,
    #[account(mut, has_one = creator @ OathError::Unauthorized,
        constraint = super::valid_oath(&oath) @ OathError::InvalidPda,
        constraint = oath.status == OathStatus::Draft @ OathError::NotDraft)]
    pub oath: Account<'info, Oath>,
    /// CHECK: canonical vault; branch-specific owner, data and authority checked below.
    #[account(mut, seeds = [b"vault", oath.key().as_ref()], bump,
        address = oath.vault @ OathError::WrongVault)]
    pub vault: UncheckedAccount<'info>,
    /// Omit for native SOL; otherwise the classic SPL mint committed by the Oath.
    #[account(address = oath.asset_mint @ OathError::WrongMint)]
    pub mint: Option<Account<'info, Mint>>,
    /// CHECK: exact creator wallet for SOL; creator-owned token account for SPL.
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CancelDraftOath>) -> Result<()> {
    let oath_key = ctx.accounts.oath.key();
    let bump_seed = [ctx.bumps.vault];
    let seeds: &[&[u8]] = &[b"vault", oath_key.as_ref(), &bump_seed];
    let signer = &[seeds];
    require_keys_neq!(ctx.accounts.destination.key(), ctx.accounts.vault.key(), OathError::WrongRecipient);

    if ctx.accounts.oath.asset_mint == Pubkey::default() {
        require!(ctx.accounts.mint.is_none(), OathError::WrongAssetType);
        require_keys_eq!(*ctx.accounts.vault.owner, system_program::ID, OathError::WrongOwner);
        require!(ctx.accounts.vault.data_is_empty(), OathError::WrongVault);
        require_keys_eq!(ctx.accounts.destination.key(), ctx.accounts.creator.key(), OathError::WrongRecipient);
        let balance = **ctx.accounts.vault.try_borrow_lamports()?;
        let reserve = Rent::get()?.minimum_balance(0);
        // An unfunded vault can be absent (zero lamports). A balance at or below
        // the reserve has no spendable escrow; cancellation must still succeed.
        let amount = if balance > reserve {
            balance.checked_sub(reserve).ok_or(OathError::ArithmeticOverflow)?
        } else {
            0
        };
        if amount > 0 {
            system_program::transfer(CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                }, signer,
            ), amount)?;
        }
    } else {
        // fund_oath_spl initializes the vault on first funding. An uninitialized
        // system-owned, empty PDA holds no SPL tokens and need not be created to cancel;
        // nothing moves, so the destination is not required to exist or be validated either
        // (a creator who never held their own launch token has no ATA for it yet).
        if *ctx.accounts.vault.owner == system_program::ID {
            require!(ctx.accounts.vault.data_is_empty(), OathError::WrongVault);
        } else {
            require_keys_eq!(*ctx.accounts.vault.owner, token::ID, OathError::WrongOwner);
            let vault_data = ctx.accounts.vault.try_borrow_data()?;
            let vault = TokenAccount::try_deserialize(&mut &vault_data[..])?;
            super::safe_token_vault(&vault, ctx.accounts.vault.key(), ctx.accounts.oath.asset_mint)?;
            let amount = vault.amount;
            drop(vault_data);
            // Sweep the actual balance, including third-party funds and direct
            // donations above commitment. Never burn or use the failure recipient.
            if amount > 0 {
                let mint = ctx.accounts.mint.as_ref().ok_or(OathError::WrongMint)?;
                require_keys_eq!(*ctx.accounts.destination.owner, token::ID, OathError::WrongOwner);
                let dest_data = ctx.accounts.destination.try_borrow_data()?;
                let destination = TokenAccount::try_deserialize(&mut &dest_data[..])?;
                require_keys_eq!(destination.mint, ctx.accounts.oath.asset_mint, OathError::WrongMint);
                require_keys_eq!(destination.owner, ctx.accounts.creator.key(), OathError::WrongRecipient);
                drop(dest_data);
                token::transfer_checked(CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(), TransferChecked {
                        from: ctx.accounts.vault.to_account_info(),
                        mint: mint.to_account_info(),
                        to: ctx.accounts.destination.to_account_info(),
                        authority: ctx.accounts.vault.to_account_info(),
                    }, signer,
                ), amount, mint.decimals)?;
            }
        }
    }
    // CPI failure rolls back the transaction; only a successful refund (or empty
    // vault) becomes terminal. Preserve all accounts as audit records/tombstones.
    ctx.accounts.oath.status = OathStatus::Cancelled;
    Ok(())
}
