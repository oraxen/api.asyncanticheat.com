//! Movement check implementations
//!
//! Categories:
//! - flight: Y prediction, sustained ascension, hover detection
//! - speed: Horizontal speed, sneak/sprint validation
//! - nofall: Invalid ground claims during fall
//! - timer: Packet rate manipulation
//! - step: Climbing too high
//! - groundspoof: Claiming ground while falling
//! - velocity: Knockback ignoring
//! - noslow: Moving too fast while sneaking/using items

pub mod flight;
pub mod speed;
pub mod nofall;
pub mod timer;
pub mod step;
pub mod groundspoof;
pub mod velocity;
pub mod noslow;

use crate::config::MovementConfig;
use crate::findings::Finding;
use crate::packets::{ParsedPacket, Location};
use crate::player_state::PlayerState;

pub use flight::FlightCheck;
pub use speed::SpeedCheck;
pub use nofall::NoFallCheck;
pub use timer::TimerCheck;
pub use step::StepCheck;
pub use groundspoof::GroundSpoofCheck;
pub use velocity::VelocityCheck;
pub use noslow::NoSlowCheck;

/// Combined movement checks processor
pub struct MovementChecks {
    flight: FlightCheck,
    speed: SpeedCheck,
    nofall: NoFallCheck,
    timer: TimerCheck,
    step: StepCheck,
    groundspoof: GroundSpoofCheck,
    velocity: VelocityCheck,
    noslow: NoSlowCheck,
}

impl MovementChecks {
    pub fn new(config: MovementConfig) -> Self {
        Self {
            flight: FlightCheck::new(config.flight.clone()),
            speed: SpeedCheck::new(config.speed.clone()),
            nofall: NoFallCheck::new(config.nofall.clone()),
            timer: TimerCheck::new(config.timer.clone()),
            step: StepCheck::new(config.step.clone()),
            groundspoof: GroundSpoofCheck::new(config.groundspoof.clone()),
            velocity: VelocityCheck::new(config.velocity.clone()),
            noslow: NoSlowCheck::new(config.noslow.clone()),
        }
    }

    /// Process a packet through all movement checks
    pub fn process(
        &self,
        state: &mut PlayerState,
        packet: &ParsedPacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Update movement state based on packet
        self.update_movement_state(state, packet, timestamp_ms);

        // Process through each check
        findings.extend(self.flight.process(state, packet, timestamp_ms));
        findings.extend(self.speed.process(state, packet, timestamp_ms));
        findings.extend(self.nofall.process(state, packet, timestamp_ms));
        findings.extend(self.timer.process(state, packet, timestamp_ms));
        findings.extend(self.step.process(state, packet, timestamp_ms));
        findings.extend(self.groundspoof.process(state, packet, timestamp_ms));
        findings.extend(self.velocity.process(state, packet, timestamp_ms));
        findings.extend(self.noslow.process(state, packet, timestamp_ms));

        findings
    }

    /// Update general movement state from packet
    fn update_movement_state(&self, state: &mut PlayerState, packet: &ParsedPacket, timestamp_ms: i64) {
        // Save previous location
        state.movement.last_location = state.movement.current_location;
        state.movement.was_on_ground = state.movement.on_ground;

        match packet {
            ParsedPacket::PositionLook(p) => {
                state.movement.current_location = Some(Location {
                    x: p.x,
                    y: p.y,
                    z: p.z,
                    yaw: p.yaw,
                    pitch: p.pitch,
                    on_ground: p.on_ground,
                });
                state.movement.on_ground = p.on_ground;
                state.movement.last_move_ms = timestamp_ms;
            }
            ParsedPacket::Position(p) => {
                let prev = state.movement.current_location.unwrap_or_default();
                state.movement.current_location = Some(Location {
                    x: p.x,
                    y: p.y,
                    z: p.z,
                    yaw: prev.yaw,
                    pitch: prev.pitch,
                    on_ground: p.on_ground,
                });
                state.movement.on_ground = p.on_ground;
                state.movement.last_move_ms = timestamp_ms;
            }
            ParsedPacket::Look(p) => {
                if let Some(ref mut loc) = state.movement.current_location {
                    loc.yaw = p.yaw;
                    loc.pitch = p.pitch;
                    loc.on_ground = p.on_ground;
                }
                state.movement.on_ground = p.on_ground;
            }
            ParsedPacket::Flying(p) => {
                if let Some(ref mut loc) = state.movement.current_location {
                    loc.on_ground = p.on_ground;
                }
                state.movement.on_ground = p.on_ground;
            }
            ParsedPacket::EntityAction(action) => {
                match action.action.as_str() {
                    "START_SPRINTING" => state.speed.is_sprinting = true,
                    "STOP_SPRINTING" => state.speed.is_sprinting = false,
                    "START_SNEAKING" | "PRESS_SHIFT_KEY" => state.speed.is_sneaking = true,
                    "STOP_SNEAKING" | "RELEASE_SHIFT_KEY" => state.speed.is_sneaking = false,
                    _ => {}
                }
            }
            ParsedPacket::UseItem(_) => {
                state.noslow.is_using_item = true;
                state.noslow.use_item_start_ms = timestamp_ms;
            }
            ParsedPacket::BlockDig(dig) => {
                // Stop using item on certain dig actions
                if dig.status == "RELEASE_USE_ITEM" || dig.status == "SWAP_HELD_ITEMS" {
                    state.noslow.is_using_item = false;
                }
            }
            _ => {}
        }

        // Decay item use after 5 seconds (max use time for most items)
        if state.noslow.is_using_item && timestamp_ms - state.noslow.use_item_start_ms > 5000 {
            state.noslow.is_using_item = false;
        }

        // Increment teleport ticks
        if state.movement.ticks_since_teleport < 100 {
            state.movement.ticks_since_teleport += 1;
        }
    }
}
