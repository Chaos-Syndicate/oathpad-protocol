use anchor_lang::prelude::*;
use crate::{errors::OathError, state::*};

#[derive(Accounts)]
pub struct ActivateOath<'info> {
    pub creator: Signer<'info>,
    #[account(mut, has_one = creator, constraint = super::valid_oath(&oath) @ OathError::InvalidPda)]
    pub oath: Account<'info, Oath>,
    #[account(seeds = [b"reviewers", oath.key().as_ref()], bump, address = oath.reviewer_set)]
    pub reviewer_set: Account<'info, ReviewerSet>,
    /// CHECK: balance, owner, authority, mint and PDA are verified before activation.
    #[account(seeds = [b"vault", oath.key().as_ref()], bump, address = oath.vault @ OathError::WrongVault)]
    pub vault: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<ActivateOath>) -> Result<()> {
    require!(ctx.accounts.oath.status == OathStatus::Draft, OathError::AlreadyActivated);
    require!(ctx.accounts.oath.milestone_count > 0, OathError::IncompleteAllocation);
    let milestones = super::read_milestones(&ctx.accounts.oath, ctx.remaining_accounts)?;
    require!(super::allocation_total(&milestones)? == ctx.accounts.oath.committed_amount, OathError::IncompleteAllocation);
    require!(super::vault_balance(&ctx.accounts.oath, &ctx.accounts.vault.to_account_info())? >= ctx.accounts.oath.committed_amount, OathError::ActivationFundingFailed);
    let now = Clock::get()?.unix_timestamp;
    for (mut milestone, info) in milestones.into_iter().zip(ctx.remaining_accounts) {
        require!(info.is_writable, OathError::Unauthorized);
        require!(milestone.status == MilestoneStatus::Pending, OathError::InvalidMilestoneState);
        require!(milestone.deadline > now, OathError::DeadlinePassed);
        require!(milestone.required_approvals == ctx.accounts.reviewer_set.threshold, OathError::InvalidThreshold);
        milestone.status = MilestoneStatus::Active;
        milestone.try_serialize(&mut &mut info.try_borrow_mut_data()?[..])?;
    }
    ctx.accounts.oath.status = OathStatus::Active;
    Ok(())
}
