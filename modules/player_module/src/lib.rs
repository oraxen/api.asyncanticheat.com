//! Player Module - Unified player behavior cheat detection
//!
//! Categories:
//! - BadPackets: Invalid packets (pitch >90, NaN position, abilities spoof, invalid slots, flying flood)
//! - Scaffold: Placing blocks while airborne or sprinting
//! - FastPlace: Placing blocks too quickly
//! - FastBreak: Breaking blocks too quickly
//! - Interact: Invalid interaction angles
//! - Inventory: Fast inventory clicks

pub mod buffer;
pub mod checks;
pub mod config;
pub mod findings;
pub mod packets;
pub mod player_state;

pub use checks::PlayerChecks;
pub use config::PlayerConfig;
pub use findings::{FeatureId, Finding};
pub use packets::ParsedPacket;
pub use player_state::PlayerState;
