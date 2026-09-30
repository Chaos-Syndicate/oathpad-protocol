pub mod protocol_config;
pub mod launch_gate;
pub mod launch_binding;
pub mod oath;
pub mod milestone;
pub mod reviewer_set;

pub use protocol_config::*;
pub use launch_gate::*;
pub use launch_binding::*;
pub use oath::*;
pub use milestone::*;
pub use reviewer_set::*;

pub const MAX_OATHS: u8 = 3;
/// Keep standalone legacy activation comfortably within the 1,232-byte limit:
/// 277 + 33 * 12 = 673 bytes (769 with a separate fee payer and signature).
/// Milestones are separate PDAs; this admission bound does not size Oath's account.
pub const MAX_MILESTONES: u8 = 12;
pub const MAX_REVIEWERS: usize = 16;
/// Bound of the dogfood allowlist in `LaunchGate`. A handful of team wallets; not a registry.
pub const MAX_ALLOWED_CREATORS: usize = 16;
