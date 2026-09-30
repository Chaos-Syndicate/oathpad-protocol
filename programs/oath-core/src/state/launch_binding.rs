use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LaunchBinding {
    pub creator: Pubkey,
    pub token_mint: Pubkey,
    pub dbc_pool: Pubkey,
    pub dbc_config: Pubkey,
    pub oath_count: u8,
    pub status: LaunchStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum LaunchStatus { Draft, OathsFunded, Live, Graduated, Closed }
