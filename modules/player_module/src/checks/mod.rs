//! Player check implementations
//!
//! Categories:
//! - badpackets: Invalid packets (pitch >90, NaN, abilities spoof, invalid slots, flying flood)
//! - scaffold: Placing blocks while airborne or sprinting
//! - fastplace: Placing blocks too quickly
//! - fastbreak: Breaking blocks too quickly
//! - interact: Invalid interaction angles
//! - inventory: Fast inventory clicks

pub mod badpackets;
pub mod scaffold;
pub mod fastplace;
pub mod fastbreak;
pub mod interact;
pub mod inventory;

use crate::config::PlayerConfig;
use crate::findings::Finding;
use crate::packets::{ParsedPacket, Location};
use crate::player_state::PlayerState;

pub use badpackets::BadPacketsCheck;
pub use scaffold::ScaffoldCheck;
pub use fastplace::FastPlaceCheck;
pub use fastbreak::FastBreakCheck;
pub use interact::InteractCheck;
pub use inventory::InventoryCheck;

/// Combined player checks processor
pub struct PlayerChecks {
    badpackets: BadPacketsCheck,
    scaffold: ScaffoldCheck,
    fastplace: FastPlaceCheck,
    fastbreak: FastBreakCheck,
    interact: InteractCheck,
    inventory: InventoryCheck,
}

impl PlayerChecks {
    pub fn new(config: PlayerConfig) -> Self {
        Self {
            badpackets: BadPacketsCheck::new(config.badpackets.clone()),
            scaffold: ScaffoldCheck::new(config.scaffold.clone()),
            fastplace: FastPlaceCheck::new(config.fastplace.clone()),
            fastbreak: FastBreakCheck::new(config.fastbreak.clone()),
            interact: InteractCheck::new(config.interact.clone()),
            inventory: InventoryCheck::new(config.inventory.clone()),
        }
    }

    /// Process a packet through all player checks
    pub fn process(
        &self,
        state: &mut PlayerState,
        packet: &ParsedPacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Process through each check
        findings.extend(self.badpackets.process(state, packet, timestamp_ms));
        findings.extend(self.scaffold.process(state, packet, timestamp_ms));
        findings.extend(self.fastplace.process(state, packet, timestamp_ms));
        findings.extend(self.fastbreak.process(state, packet, timestamp_ms));
        findings.extend(self.interact.process(state, packet, timestamp_ms));
        findings.extend(self.inventory.process(state, packet, timestamp_ms));

        // Update movement state based on packet type
        match packet {
            ParsedPacket::Look(look) => {
                state.movement.last_yaw = look.yaw;
                state.movement.last_pitch = look.pitch;
                state.movement.on_ground = look.on_ground;
            }
            ParsedPacket::PositionLook(pos_look) => {
                state.movement.last_location = Some(Location {
                    x: pos_look.x,
                    y: pos_look.y,
                    z: pos_look.z,
                    yaw: pos_look.yaw,
                    pitch: pos_look.pitch,
                    on_ground: pos_look.on_ground,
                });
                state.movement.last_yaw = pos_look.yaw;
                state.movement.last_pitch = pos_look.pitch;
                state.movement.on_ground = pos_look.on_ground;
                state.movement.last_move_ms = timestamp_ms;
            }
            ParsedPacket::Position(pos) => {
                if let Some(ref mut loc) = state.movement.last_location {
                    loc.x = pos.x;
                    loc.y = pos.y;
                    loc.z = pos.z;
                    loc.on_ground = pos.on_ground;
                } else {
                    state.movement.last_location = Some(Location {
                        x: pos.x,
                        y: pos.y,
                        z: pos.z,
                        yaw: state.movement.last_yaw,
                        pitch: state.movement.last_pitch,
                        on_ground: pos.on_ground,
                    });
                }
                state.movement.on_ground = pos.on_ground;
                state.movement.last_move_ms = timestamp_ms;
            }
            ParsedPacket::Flying(flying) => {
                state.movement.on_ground = flying.on_ground;
            }
            ParsedPacket::EntityAction(action) => {
                if action.action.contains("SPRINT") {
                    state.scaffold.is_sprinting = action.action.contains("START");
                }
            }
            _ => {}
        }

        findings
    }
}
