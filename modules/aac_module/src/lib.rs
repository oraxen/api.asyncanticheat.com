//! AAC5-style anti-cheat module for AsyncAnticheat
//!
//! This module implements checks inspired by AAC5 (Advanced Anti-Cheat):
//! - Delays: Actions performed faster than normally possible
//! - Move: Timer, NoFall, Velocity, NoSlow, etc.
//! - Aimbot: Sensitivity mismatches, head snaps, pitch spread
//! - Autoclicker: Click patterns, timing consistency
//! - Hitbox: Reach, line-of-sight, hit patterns
//! - Interact: Invalid block interactions
//! - Misc: Invalid pitch, rotation rate

pub mod checks;
pub mod config;
pub mod findings;
pub mod packets;
pub mod player_state;
pub mod vl;

pub use config::AacConfig;
pub use findings::{FeatureId, Finding, Severity};
pub use player_state::PlayerState;
pub use vl::ViolationLevel;
