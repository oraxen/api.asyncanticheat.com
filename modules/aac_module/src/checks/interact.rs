//! Interact check (ce_0) - Abnormal world interactions
//!
//! Detects:
//! - Right-click block interactions where aim is too far from target block face
//! - Impossible placement/break interaction angles

use crate::config::InteractConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::{BlockPlacePacket, ParsedPacket};
use crate::player_state::PlayerState;

/// Maximum interaction limit (static int g = 5 in ce_0)
const INTERACTION_LIMIT: u32 = 5;

pub struct InteractCheck {
    config: InteractConfig,
}

impl InteractCheck {
    pub fn new(config: InteractConfig) -> Self {
        Self { config }
    }

    /// Process a packet and return any findings
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

        match packet {
            ParsedPacket::BlockPlace(place) => {
                findings.extend(self.check_block_place(state, place, timestamp_ms));
            }
            _ => {}
        }

        findings
    }

    fn check_block_place(
        &self,
        state: &mut PlayerState,
        place: &BlockPlacePacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Get player's current look direction
        if let Some(loc) = state.movement.last_location {
            // Calculate expected look direction to target block
            let player_x = loc.x;
            let player_y = loc.y + 1.62; // Eye height
            let player_z = loc.z;

            let block_center_x = place.x as f64 + 0.5;
            let block_center_y = place.y as f64 + 0.5;
            let block_center_z = place.z as f64 + 0.5;

            let dx = block_center_x - player_x;
            let dy = block_center_y - player_y;
            let dz = block_center_z - player_z;

            let horizontal_dist = (dx * dx + dz * dz).sqrt();
            let expected_yaw = (-dx.atan2(dz)).to_degrees() as f32;
            let expected_pitch = (-dy.atan2(horizontal_dist)).to_degrees() as f32;

            // Calculate angle difference
            let yaw_diff = self.normalize_angle(loc.yaw - expected_yaw).abs();
            let pitch_diff = (loc.pitch - expected_pitch).abs();
            
            let angle_diff = ((yaw_diff.powi(2) + pitch_diff.powi(2)) as f64).sqrt();
            let angle_diff_rad = angle_diff.to_radians();

            // Check if angle difference exceeds threshold
            if angle_diff_rad > self.config.max_angle_diff as f64 {
                state.interact.invalid_interactions += 1;

                if state.interact.invalid_interactions >= INTERACTION_LIMIT {
                    let mitigated = state.interact.vl.update(1.0, timestamp_ms);

                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::AacInteractPlace,
                            angle_diff as f32,
                            state.interact.vl.get(),
                            mitigated,
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "Block placed at {:.1}° from look direction (max: {:.1}°)",
                            angle_diff,
                            self.config.max_angle_diff.to_degrees()
                        )),
                    );
                }
            } else {
                // Reset counter on valid interaction
                if state.interact.invalid_interactions > 0 {
                    state.interact.invalid_interactions -= 1;
                }
            }
        }

        state.interact.last_interact_ms = timestamp_ms;
        findings
    }

    fn normalize_angle(&self, angle: f32) -> f32 {
        let mut a = angle;
        while a > 180.0 {
            a -= 360.0;
        }
        while a < -180.0 {
            a += 360.0;
        }
        a
    }
}

