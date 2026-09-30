use anchor_lang::prelude::*;
use crate::{errors::OathError, state::*};

#[derive(Accounts)]
pub struct CloseCompletedOath<'info> {
    #[account(mut, constraint = super::valid_oath(&oath) @ OathError::InvalidPda)]
    pub oath: Account<'info, Oath>,
}

pub fn handler(ctx: Context<CloseCompletedOath>) -> Result<()> {
    require!(ctx.accounts.oath.status == OathStatus::Active, OathError::NotActive);
    let milestones = super::read_milestones(&ctx.accounts.oath, ctx.remaining_accounts)?;
    for milestone in milestones {
        require!(matches!(milestone.status, MilestoneStatus::Claimed | MilestoneStatus::ConsequenceExecuted), OathError::UnsettledMilestone);
    }
    // Preserve audit state and PDA tombstones. Rent and unsolicited donations
    // are not a back door for withdrawing committed escrow.
    ctx.accounts.oath.status = OathStatus::Completed;
    Ok(())
}
