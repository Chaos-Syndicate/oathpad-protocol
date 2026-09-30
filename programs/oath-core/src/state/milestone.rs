use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Milestone {
    pub oath: Pubkey,
    pub index: u8,
    pub allocation_amount: u64,
    pub deadline: i64,
    pub evidence_hash: [u8; 32],
    pub approvals_count: u8,
    pub required_approvals: u8,
    pub status: MilestoneStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum MilestoneStatus {
    Pending, Active, EvidenceSubmitted, Approved, Claimable, Claimed,
    Expired, Failed, ConsequenceExecuted,
}
