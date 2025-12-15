//! Vulcan-style anti-cheat module for AsyncAnticheat
//!
//! Based on Vulcan 2.9.7.8 analysis. Implements checks in three categories:
//!
//! ## Combat Checks
//! - Aim: Rotation analysis, GCD detection, sensitivity checks
//! - Auto Block: Block/attack packet ordering
//! - Auto Clicker: Statistical click analysis (CPS, variance, kurtosis)
//! - Criticals: Ground state validation during attacks
//! - Hitbox: Angle-based hitbox validation
//! - Kill Aura: Combat automation detection
//! - Reach: Distance-based attack validation
//! - Velocity: Knockback response validation
//!
//! ## Movement Checks
//! - Flight: Y-axis prediction and hover detection
//! - Speed: Friction and ground speed validation
//! - No Slow: Slowdown enforcement (item use, soul sand, web)
//! - Jump: Motion validation
//! - Jesus: Water walking detection
//! - Elytra: Gliding physics validation
//! - Step: Step height enforcement
//! - Timer: Game speed manipulation
//!
//! ## Player Checks
//! - Bad Packets: Protocol violations
//! - Scaffold: Automated bridging detection
//! - Fast Break/Place: Block interaction timing
//! - Ground Spoof: Falsified ground state
//! - Invalid: Position sanity checks

pub mod checks;
pub mod config;
pub mod findings;
pub mod packets;
pub mod player_state;
pub mod buffer;

pub use buffer::CheckBuffer;
pub use config::VulcanConfig;
pub use findings::{FeatureId, Finding, Severity};
pub use player_state::PlayerState;

