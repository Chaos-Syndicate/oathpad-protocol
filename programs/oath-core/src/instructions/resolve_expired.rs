use anchor_lang::prelude::*;
use crate::{errors::OathError, state::*};

#[derive(Accounts)]
pub struct ResolveExpired<'info> {
    #[account(constraint = super::valid_oath(&oath) @ OathError::InvalidPda)]
    pub oath: Account<'info, Oath>,
    #[account(mut, seeds = [b"milestone", oath.key().as_ref(), &[milestone.index]], bump, has_one = oath)]
    pub milestone: Account<'info, Milestone>,
}

pub fn handler(ctx: Context<ResolveExpired>) -> Result<()> {
    require!(ctx.accounts.oath.status == OathStatus::Active, OathError::NotActive);
    let milestone = &mut ctx.accounts.milestone;
    require!(matches!(milestone.status, MilestoneStatus::Active | MilestoneStatus::EvidenceSubmitted), OathError::TerminalMilestone);
    require!(Clock::get()?.unix_timestamp > milestone.deadline, OathError::DeadlineNotPassed);
    milestone.status = MilestoneStatus::Expired;
    Ok(())
}
