//! Scaffold detection
//!
//! Detects:
//! - Placing blocks on bottom face while airborne (classic scaffold)
//! - Sprinting while bridging

use crate::config::ScaffoldConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

/// Block face for bottom (placing under player)
const FACE_BOTTOM: i32 = 1;
/// Minimum interval for scaffold detection (ms)
const SCAFFOLD_INTERVAL_MS: i64 = 500;

pub struct ScaffoldCheck {
    config: ScaffoldConfig,
}

impl ScaffoldCheck {
    pub fn new(config: ScaffoldConfig) -> Self {
        Self { config }
    }

    pub fn process(
        &self,
        state: &mut PlayerState,
        packet: &ParsedPacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        if !self.config.enabled {
            return Vec::new();
        }

        let mut findings = Vec::new();

        if let ParsedPacket::BlockPlace(place) = packet {
            findings.extend(self.check_scaffold(state, place.face, timestamp_ms));
        }

        findings
    }

    fn check_scaffold(
        &self,
        state: &mut PlayerState,
        face: i32,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        let is_bottom_place = face == FACE_BOTTOM;
        let is_airborne = !state.movement.on_ground;
        let interval = timestamp_ms - state.scaffold.last_place_ms;

        // Check for airborne bottom-face placement (scaffold pattern)
        if self.config.check_airborne_bottom && is_bottom_place && is_airborne {
            // Consecutive scaffold placement
            if state.scaffold.last_place_face == FACE_BOTTOM && interval < SCAFFOLD_INTERVAL_MS {
                state.scaffold.consecutive_scaffold += 1;
            } else {
                state.scaffold.consecutive_scaffold = 1;
            }

            if state.scaffold.consecutive_scaffold >= self.config.min_scaffold_count {
                let flagged = state.scaffold.buffer_airborne.fail();
                if flagged {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::ScaffoldAirborne,
                            state.scaffold.consecutive_scaffold as f64,
                            state.scaffold.buffer_airborne.vl(),
                            state.scaffold.buffer_airborne.max_vl(),
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "Scaffold: {} consecutive bottom-face placements while airborne",
                            state.scaffold.consecutive_scaffold
                        )),
                    );
                }
            }
        } else if is_bottom_place {
            state.scaffold.buffer_airborne.pass();
        }

        // Check for sprinting while bridging
        if self.config.check_sprint_bridge && is_bottom_place && state.scaffold.is_sprinting {
            let flagged = state.scaffold.buffer_sprint.fail();
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::ScaffoldSprint,
                        1.0,
                        state.scaffold.buffer_sprint.vl(),
                        state.scaffold.buffer_sprint.max_vl(),
                        timestamp_ms,
                    )
                    .with_description("Scaffold: sprinting while bridging".to_string()),
                );
            }
        } else if is_bottom_place {
            state.scaffold.buffer_sprint.pass();
        }

        // Reset consecutive if not bottom face or large gap
        if !is_bottom_place || interval > SCAFFOLD_INTERVAL_MS {
            state.scaffold.consecutive_scaffold = 0;
        }

        state.scaffold.last_place_ms = timestamp_ms;
        state.scaffold.last_place_face = face;

        findings
    }
}
