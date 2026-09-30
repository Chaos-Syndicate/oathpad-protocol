use anchor_lang::{prelude::*, system_program};
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, TransferChecked};
use crate::{errors::OathError, state::*};

pub mod initialize_protocol;
pub mod launch_gate;
pub mod create_launch_binding;
pub mod create_oath;
pub mod create_reviewer_set;
pub mod add_milestone;
pub mod fund_oath_spl;
pub mod fund_oath_sol;
pub mod activate_oath;
pub mod cancel_draft_oath;
pub mod submit_evidence;
pub mod approve_milestone;
pub mod finalize_milestone;
pub mod claim_tranche;
pub mod resolve_expired;
pub mod execute_failure;
pub mod close_completed_oath;

pub use initialize_protocol::*;
pub use launch_gate::*;
pub use create_launch_binding::*;
pub use create_oath::*;
pub use create_reviewer_set::*;
pub use add_milestone::*;
pub use fund_oath_spl::*;
pub use fund_oath_sol::*;
pub use activate_oath::*;
pub use cancel_draft_oath::*;
pub use submit_evidence::*;
pub use approve_milestone::*;
pub use finalize_milestone::*;
pub use resolve_expired::*;
pub use close_completed_oath::*;

/// The index is intentionally not an extra Oath field: schema fixes its fields.
/// Verify membership in the bounded three-PDA namespace on every Oath access.
pub fn valid_oath(oath: &Account<Oath>) -> bool {
    (0..MAX_OATHS).any(|index| {
        Pubkey::find_program_address(&[b"oath", oath.launch.as_ref(), &[index]], &crate::ID).0 == oath.key()
    })
}

pub fn read_milestones(oath: &Account<Oath>, accounts: &[AccountInfo]) -> Result<Vec<Milestone>> {
    require!(accounts.len() == usize::from(oath.milestone_count), OathError::IncompleteMilestoneList);
    let mut result = Vec::with_capacity(accounts.len());
    for (index, info) in accounts.iter().enumerate() {
        require_keys_eq!(*info.owner, crate::ID, OathError::WrongOwner);
        let expected = Pubkey::find_program_address(&[b"milestone", oath.key().as_ref(), &[index as u8]], &crate::ID).0;
        require_keys_eq!(info.key(), expected, OathError::InvalidPda);
        let data = info.try_borrow_data()?;
        let milestone = Milestone::try_deserialize(&mut &data[..])?;
        require_keys_eq!(milestone.oath, oath.key(), OathError::InvalidPda);
        require!(usize::from(milestone.index) == index, OathError::InvalidPda);
        result.push(milestone);
    }
    Ok(result)
}

pub fn allocation_total(milestones: &[Milestone]) -> Result<u64> {
    milestones.iter().try_fold(0u64, |sum, m| sum.checked_add(m.allocation_amount).ok_or_else(|| error!(OathError::ArithmeticOverflow)))
}

/// A funding request authorizes at most `maximum`, against the live vault balance.
/// Direct donations may already have met or exceeded the cap; never add to that surplus.
pub fn funding_amount(balance: u64, cap: u64, maximum: u64) -> Result<u64> {
    if balance >= cap {
        return Ok(0);
    }
    let remaining = cap.checked_sub(balance).ok_or(OathError::ArithmeticOverflow)?;
    Ok(maximum.min(remaining))
}

pub fn safe_token_vault(vault: &TokenAccount, key: Pubkey, mint: Pubkey) -> Result<()> {
    require_keys_eq!(vault.owner, key, OathError::WrongVault);
    require_keys_eq!(vault.mint, mint, OathError::WrongMint);
    require!(vault.delegate.is_none() && vault.close_authority.is_none(), OathError::UnsafeVault);
    require!(vault.state == token::spl_token::state::AccountState::Initialized && vault.is_native.is_none(), OathError::UnsafeVault);
    Ok(())
}

pub fn vault_balance(oath: &Oath, vault: &AccountInfo) -> Result<u64> {
    require_keys_eq!(vault.key(), oath.vault, OathError::WrongVault);
    if oath.asset_mint == Pubkey::default() {
        require_keys_eq!(*vault.owner, system_program::ID, OathError::WrongOwner);
        require!(vault.data_is_empty(), OathError::WrongVault);
        Ok(vault.lamports().saturating_sub(Rent::get()?.minimum_balance(0)))
    } else {
        require_keys_eq!(*vault.owner, token::ID, OathError::WrongOwner);
        let data = vault.try_borrow_data()?;
        let parsed = TokenAccount::try_deserialize(&mut &data[..])?;
        safe_token_vault(&parsed, vault.key(), oath.asset_mint)?;
        Ok(parsed.amount)
    }
}

#[derive(Accounts)]
pub struct Settlement<'info> {
    /// Creator for claims; any signer for failure execution.
    pub caller: Signer<'info>,
    #[account(constraint = valid_oath(&oath) @ OathError::InvalidPda)]
    pub oath: Account<'info, Oath>,
    #[account(mut, seeds = [b"milestone", oath.key().as_ref(), &[milestone.index]], bump, has_one = oath)]
    pub milestone: Account<'info, Milestone>,
    /// CHECK: canonical PDA; branch-specific owner and balance validated by settle.
    #[account(mut, seeds = [b"vault", oath.key().as_ref()], bump, address = oath.vault @ OathError::WrongVault)]
    pub vault: UncheckedAccount<'info>,
    /// Omit for native SOL; classic SPL owner and stored mint checked by Anchor.
    #[account(mut, address = oath.asset_mint @ OathError::WrongMint)]
    pub mint: Option<Account<'info, Mint>>,
    /// CHECK: exact SOL recipient or recipient's canonical SPL ATA, validated by settle.
    #[account(mut)] pub destination: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn settle<'info>(accounts: &Settlement<'info>, recipient: Pubkey, burn: bool, bump: u8) -> Result<()> {
    let amount = accounts.milestone.allocation_amount;
    require!(amount > 0 && amount <= accounts.oath.committed_amount, OathError::AllocationExceedsCommitted);
    require!(vault_balance(&accounts.oath, &accounts.vault.to_account_info())? >= amount, OathError::InsufficientVaultBalance);
    let key = accounts.oath.key();
    let bump_seed = [bump];
    let seeds: &[&[u8]] = &[b"vault", key.as_ref(), &bump_seed];
    let signer = &[seeds];
    if accounts.oath.asset_mint == Pubkey::default() {
        require!(accounts.mint.is_none(), OathError::WrongAssetType);
        require!(!burn, OathError::UnsupportedSolBurn);
        require_keys_eq!(accounts.destination.key(), recipient, OathError::WrongRecipient);
        require_keys_neq!(accounts.destination.key(), accounts.vault.key(), OathError::WrongRecipient);
        system_program::transfer(CpiContext::new_with_signer(accounts.system_program.to_account_info(), system_program::Transfer {
            from: accounts.vault.to_account_info(), to: accounts.destination.to_account_info(),
        }, signer), amount)?;
    } else {
        let mint = accounts.mint.as_ref().ok_or(OathError::WrongMint)?;
        let decimals = mint.decimals;
        if burn {
            token::burn(CpiContext::new_with_signer(accounts.token_program.to_account_info(), Burn {
                mint: mint.to_account_info(), from: accounts.vault.to_account_info(), authority: accounts.vault.to_account_info(),
            }, signer), amount)?;
        } else {
            require_keys_neq!(accounts.destination.key(), accounts.vault.key(), OathError::WrongRecipient);
            // Applies to fixed-recipient consequences and creator claims alike.
            // Derive from the immutable recipient and classic SPL mint, including PDA recipients.
            let expected = get_associated_token_address(&recipient, &accounts.oath.asset_mint);
            require_keys_eq!(accounts.destination.key(), expected, OathError::WrongRecipient);
            // Callers may prepend idempotent ATA creation in the same transaction.
            // A missing (or merely SOL-prefunded) ATA has no token-account data yet.
            require!(!accounts.destination.data_is_empty(), OathError::RecipientAtaMissing);
            require_keys_eq!(*accounts.destination.owner, token::ID, OathError::WrongOwner);
            let data = accounts.destination.try_borrow_data()?;
            let destination = TokenAccount::try_deserialize(&mut &data[..])?;
            require_keys_eq!(destination.mint, accounts.oath.asset_mint, OathError::WrongMint);
            require_keys_eq!(destination.owner, recipient, OathError::WrongRecipient);
            drop(data);
            token::transfer_checked(CpiContext::new_with_signer(accounts.token_program.to_account_info(), TransferChecked {
                from: accounts.vault.to_account_info(), mint: mint.to_account_info(),
                to: accounts.destination.to_account_info(), authority: accounts.vault.to_account_info(),
            }, signer), amount, decimals)?;
        }
    }
    Ok(())
}
