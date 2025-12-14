//! AAC5-style check implementations
//!
//! Each module corresponds to a check from AAC5:
//! - delays (cm_0): Fast actions
//! - movement (ca_0): Movement validation
//! - aimbot (c7): Aim modifications
//! - autoclicker (cr_0): Click patterns
//! - hitbox (ch_0): Combat reach
//! - interact (ce_0): Block interactions
//! - misc (cO + cC): Miscellaneous checks

pub mod delays;
pub mod movement;
pub mod aimbot;
pub mod autoclicker;
pub mod hitbox;
pub mod interact;
pub mod misc;

pub use delays::DelaysCheck;
pub use movement::MovementCheck;
pub use aimbot::AimbotCheck;
pub use autoclicker::AutoclickerCheck;
pub use hitbox::HitboxCheck;
pub use interact::InteractCheck;
pub use misc::MiscCheck;

