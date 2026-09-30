use anchor_lang::prelude::*;
use crate::state::MAX_ALLOWED_CREATORS;

/// Who may create a `LaunchBinding` or an `Oath` while the protocol is in a closed
/// (team dogfood) phase. Singleton PDA `["launch_gate"]`, managed only by
/// `ProtocolConfig.authority`.
///
/// It is a sibling of `ProtocolConfig` rather than new fields on it because the
/// 73-byte `ProtocolConfig` already exists on devnet and has no padding; extending it
/// would need a resize migration of a live account.
///
/// Scope: creation only. No funding, activation, evidence, approval, claim, expiry,
/// failure or close path reads this account, so gate management can never reach a
/// creator's locked assets or any post-activation-immutable field
/// (docs/THREAT-MODEL.md section 4).
#[account]
#[derive(InitSpace)]
pub struct LaunchGate {
    /// When true the allowlist is ignored and any creator may create launches and Oaths.
    pub launches_permissionless: bool,
    /// Creators allowed while `launches_permissionless == false`. Bounded, no duplicates,
    /// never the zero key.
    #[max_len(MAX_ALLOWED_CREATORS)]
    pub allowed_creators: Vec<Pubkey>,
}

impl LaunchGate {
    /// The gate check: `!launches_permissionless => creator must be allowlisted`.
    pub fn allows(&self, creator: &Pubkey) -> bool {
        self.launches_permissionless || self.allowed_creators.contains(creator)
    }
}
