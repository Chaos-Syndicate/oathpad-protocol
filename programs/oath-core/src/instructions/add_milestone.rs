use anchor_lang::prelude::*;
use crate::{errors::OathError, state::*};

#[derive(Accounts)]
pub struct AddMilestone<'info> {
    #[account(mut)] pub creator: Signer<'info>,
    #[account(mut, has_one = creator, constraint = super::valid_oath(&oath) @ OathError::InvalidPda)]
    pub oath: Account<'info, Oath>,
    #[account(seeds = [b"reviewers", oath.key().as_ref()], bump, address = oath.reviewer_set)]
    pub reviewer_set: Account<'info, ReviewerSet>,
    #[account(init, payer = creator, space = 8 + Milestone::INIT_SPACE,
        seeds = [b"milestone", oath.key().as_ref(), &[oath.milestone_count]], bump)]
    pub milestone: Account<'info, Milestone>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddMilestone>, allocation_amount: u64, deadline: i64) -> Result<()> {
    let oath = &mut ctx.accounts.oath;
    require!(oath.status == OathStatus::Draft, OathError::MutationAfterActivation);
    require!(oath.milestone_count < MAX_MILESTONES, OathError::TooManyMilestones);
    require!(allocation_amount > 0, OathError::ZeroAmount);
    require!(deadline > Clock::get()?.unix_timestamp, OathError::InvalidDeadline);
    let existing = super::read_milestones(oath, ctx.remaining_accounts)?;
    let total = super::allocation_total(&existing)?.checked_add(allocation_amount).ok_or(OathError::ArithmeticOverflow)?;
    require!(total <= oath.committed_amount, OathError::AllocationExceedsCommitted);
    ctx.accounts.milestone.set_inner(Milestone {
        oath: oath.key(), index: oath.milestone_count, allocation_amount, deadline,
        evidence_hash: [0; 32], approvals_count: 0,
        required_approvals: ctx.accounts.reviewer_set.threshold, status: MilestoneStatus::Pending,
    });
    oath.milestone_count = oath.milestone_count.checked_add(1).ok_or(OathError::ArithmeticOverflow)?;
    Ok(())
}
