//! Movement Module - Unified movement cheat detection
//!
//! Categories:
//! - Flight: Detecting flying (Y prediction, sustained ascension, hover)
//! - Speed: Detecting speed hacks (horizontal speed, sneak/sprint)
//! - NoFall: Detecting NoFall (invalid ground claims during fall)
//! - Timer: Detecting timer cheats (packet rate manipulation)
//! - Step: Detecting step hacks (climbing too high)
//! - GroundSpoof: Detecting ground spoofing (claiming ground while falling)
//! - Velocity: Detecting velocity/knockback ignoring
//! - NoSlow: Detecting NoSlow (moving too fast while sneaking/using items)

pub mod checks;
pub mod config;
pub mod findings;
pub mod packets;
pub mod player_state;
pub mod buffer;

pub use checks::MovementChecks;
pub use config::MovementConfig;
pub use findings::{FeatureId, Finding};
pub use packets::ParsedPacket;
pub use player_state::PlayerState;
