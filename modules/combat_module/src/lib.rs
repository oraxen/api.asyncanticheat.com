//! Combat Module - Unified combat cheat detection
//!
//! Categories:
//! - KillAura: Multi-target attacks, post-attack timing
//! - Aim: Rotation analysis, GCD, snapping, patterns
//! - AutoClicker: CPS, timing statistics, variance analysis
//! - Reach: Attack distance validation
//! - NoSwing: Attack without arm animation

pub mod checks;
pub mod config;
pub mod findings;
pub mod packets;
pub mod player_state;
pub mod buffer;

pub use checks::CombatChecks;
pub use config::CombatConfig;
pub use findings::{FeatureId, Finding};
pub use packets::ParsedPacket;
pub use player_state::PlayerState;
