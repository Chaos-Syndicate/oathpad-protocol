use anchor_lang::prelude::*;
use crate::{errors::OathError, state::*};
pub use super::Settlement as ClaimTranche;

pub fn handler(ctx: Context<ClaimTranche>) -> Result<()> {
    require!(ctx.accounts.oath.status == OathStatus::Active, OathError::NotActive);
    require_keys_eq!(ctx.accounts.caller.key(), ctx.accounts.oath.creator, OathError::Unauthorized);
    require!(ctx.accounts.milestone.status != MilestoneStatus::Claimed, OathError::AlreadyClaimed);
    require!(ctx.accounts.milestone.approvals_count >= ctx.accounts.milestone.required_approvals, OathError::NotApproved);
    require!(ctx.accounts.milestone.status == MilestoneStatus::Claimable, OathError::NotClaimable);
    super::settle(&ctx.accounts, ctx.accounts.oath.creator, false, ctx.bumps.vault)?;
    ctx.accounts.milestone.status = MilestoneStatus::Claimed;
    Ok(())
}
