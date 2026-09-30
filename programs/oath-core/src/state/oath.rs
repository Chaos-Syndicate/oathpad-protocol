use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Oath {
    pub launch: Pubkey,
    pub creator: Pubkey,
    /// Pubkey::default() denotes native SOL; all other values are classic SPL mints.
    pub asset_mint: Pubkey,
    pub vault: Pubkey,
    pub committed_amount: u64,
    pub milestone_count: u8,
    pub reviewer_set: Pubkey,
    pub failure_action: FailureAction,
    pub status: OathStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum FailureAction { Burn, FixedRecipient { recipient: Pubkey } }

// Append variants to preserve the deployed Borsh tags and account size.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum OathStatus { Draft, Active, Completed, Cancelled }
