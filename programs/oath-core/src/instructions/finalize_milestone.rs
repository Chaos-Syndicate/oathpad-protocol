use anchor_lang::prelude::*;
use crate::{errors::OathError, state::*};

#[derive(Accounts)]
pub struct FinalizeMilestone<'info> {
    #[account(constraint = super::valid_oath(&oath) @ OathError::InvalidPda)]
    pub oath: Account<'info, Oath>,
    #[account(mut, seeds = [b"milestone", oath.key().as_ref(), &[milestone.index]], bump, has_one = oath)]
    pub milestone: Account<'info, Milestone>,
}

pub fn handler(ctx: Context<FinalizeMilestone>) -> Result<()> {
    require!(ctx.accounts.oath.status == OathStatus::Active, OathError::NotActive);
    let milestone = &mut ctx.accounts.milestone;
    require!(milestone.status == MilestoneStatus::Approved && milestone.approvals_count >= milestone.required_approvals, OathError::NotApproved);
    milestone.status = MilestoneStatus::Claimable;
    Ok(())
}
