use anchor_lang::prelude::*;
use crate::{errors::OathError, state::*};

/// Never closed: the PDA is the immutable per-milestone vote uniqueness record.
#[account]
#[derive(InitSpace)]
pub struct ApprovalReceipt { pub approved: bool }

#[derive(Accounts)]
pub struct ApproveMilestone<'info> {
    #[account(mut)] pub reviewer: Signer<'info>,
    #[account(constraint = super::valid_oath(&oath) @ OathError::InvalidPda)]
    pub oath: Account<'info, Oath>,
    #[account(seeds = [b"reviewers", oath.key().as_ref()], bump, address = oath.reviewer_set)]
    pub reviewer_set: Account<'info, ReviewerSet>,
    #[account(mut, seeds = [b"milestone", oath.key().as_ref(), &[milestone.index]], bump, has_one = oath)]
    pub milestone: Account<'info, Milestone>,
    #[account(init_if_needed, payer = reviewer, space = 8 + ApprovalReceipt::INIT_SPACE,
        seeds = [b"approval", milestone.key().as_ref(), reviewer.key().as_ref()], bump)]
    pub receipt: Account<'info, ApprovalReceipt>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ApproveMilestone>) -> Result<()> {
    require_keys_neq!(ctx.accounts.reviewer.key(), ctx.accounts.oath.creator, OathError::SelfApproval);
    require!(ctx.accounts.reviewer_set.reviewers.contains(&ctx.accounts.reviewer.key()), OathError::WrongReviewer);
    require!(!ctx.accounts.receipt.approved, OathError::AlreadyApproved);
    require!(ctx.accounts.oath.status == OathStatus::Active, OathError::NotActive);
    let milestone = &mut ctx.accounts.milestone;
    require!(milestone.status == MilestoneStatus::EvidenceSubmitted, OathError::InvalidMilestoneState);
    require!(Clock::get()?.unix_timestamp <= milestone.deadline, OathError::DeadlinePassed);
    require!(milestone.required_approvals == ctx.accounts.reviewer_set.threshold, OathError::InvalidThreshold);
    milestone.approvals_count = milestone.approvals_count.checked_add(1).ok_or(OathError::ArithmeticOverflow)?;
    ctx.accounts.receipt.approved = true;
    if milestone.approvals_count >= milestone.required_approvals {
        milestone.status = MilestoneStatus::Approved;
    }
    Ok(())
}
