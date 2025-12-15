//! Vulcan-style check implementations
//!
//! Organized into three categories:
//! - combat: Aim, Auto Clicker, Velocity, Reach, Hitbox, Kill Aura, Criticals
//! - movement: Flight, Speed, No Slow, Jump, Timer, Ground Spoof
//! - player: Bad Packets, Scaffold, Fast Break, Fast Place

pub mod combat;
pub mod movement;
pub mod player;

pub use combat::CombatChecks;
pub use movement::MovementChecks;
pub use player::PlayerChecks;

