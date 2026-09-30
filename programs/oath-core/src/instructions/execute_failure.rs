use anchor_lang::prelude::*;
use crate::{errors::OathError, state::*};
use super::Settlement;

pub fn handler(ctx: Context<Settlement>) -> Result<()> {
    require!(ctx.accounts.oath.status == OathStatus::Active, OathError::NotActive);
    require!(matches!(ctx.accounts.milestone.status, MilestoneStatus::Expired | MilestoneStatus::Failed), OathError::FailureNotReady);
    require!(Clock::get()?.unix_timestamp > ctx.accounts.milestone.deadline, OathError::DeadlineNotPassed);
    match ctx.accounts.oath.failure_action {
        FailureAction::Burn => super::settle(&ctx.accounts, Pubkey::default(), true, ctx.bumps.vault)?,
        FailureAction::FixedRecipient { recipient } => super::settle(&ctx.accounts, recipient, false, ctx.bumps.vault)?,
    }
    ctx.accounts.milestone.status = MilestoneStatus::ConsequenceExecuted;
    Ok(())
}
