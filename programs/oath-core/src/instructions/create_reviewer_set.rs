use anchor_lang::prelude::*;
use crate::{errors::OathError, state::*};

#[derive(Accounts)]
pub struct CreateReviewerSet<'info> {
    #[account(mut)] pub creator: Signer<'info>,
    #[account(has_one = creator, constraint = super::valid_oath(&oath) @ OathError::InvalidPda)]
    pub oath: Account<'info, Oath>,
    #[account(init, payer = creator, space = 8 + ReviewerSet::INIT_SPACE,
        seeds = [b"reviewers", oath.key().as_ref()], bump, address = oath.reviewer_set)]
    pub reviewer_set: Account<'info, ReviewerSet>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateReviewerSet>, reviewers: Vec<Pubkey>, threshold: u8) -> Result<()> {
    require!(ctx.accounts.oath.status == OathStatus::Draft, OathError::MutationAfterActivation);
    require!(reviewers.len() <= MAX_REVIEWERS, OathError::TooManyReviewers);
    require!(threshold > 0 && usize::from(threshold) <= reviewers.len(), OathError::InvalidThreshold);
    for (i, reviewer) in reviewers.iter().enumerate() {
        require_keys_neq!(*reviewer, ctx.accounts.creator.key(), OathError::SelfApproval);
        require_keys_neq!(*reviewer, Pubkey::default(), OathError::InvalidReviewer);
        require!(!reviewers[..i].contains(reviewer), OathError::DuplicateReviewer);
    }
    ctx.accounts.reviewer_set.set_inner(ReviewerSet { reviewers, threshold });
    Ok(())
}
