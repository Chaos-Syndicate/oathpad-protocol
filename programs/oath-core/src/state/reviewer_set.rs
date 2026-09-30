use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ReviewerSet {
    #[max_len(16)]
    pub reviewers: Vec<Pubkey>,
    pub threshold: u8,
}
