use anchor_lang::prelude::*;
use crate::{errors::OathError, state::*};

#[derive(Accounts)]
pub struct SubmitEvidence<'info> {
    pub creator: Signer<'info>,
    #[account(has_one = creator, constraint = super::valid_oath(&oath) @ OathError::InvalidPda)]
    pub oath: Account<'info, Oath>,
    #[account(mut, seeds = [b"milestone", oath.key().as_ref(), &[milestone.index]], bump, has_one = oath)]
    pub milestone: Account<'info, Milestone>,
}

pub fn handler(ctx: Context<SubmitEvidence>, evidence_hash: [u8; 32]) -> Result<()> {
    require!(ctx.accounts.oath.status == OathStatus::Active, OathError::NotActive);
    let milestone = &mut ctx.accounts.milestone;
    require!(milestone.status == MilestoneStatus::Active, OathError::InvalidMilestoneState);
    require!(Clock::get()?.unix_timestamp <= milestone.deadline, OathError::DeadlinePassed);
    require!(evidence_hash != [0; 32] && milestone.evidence_hash == [0; 32], OathError::InvalidEvidence);
    milestone.evidence_hash = evidence_hash;
    milestone.status = MilestoneStatus::EvidenceSubmitted;
    Ok(())
}
