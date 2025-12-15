//! Interact detection
//!
//! Detects:
//! - Invalid interaction angles (looking away from block)
//! - Impossible interaction angles (through walls, etc.)

use crate::config::InteractConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

pub struct InteractCheck {
    config: InteractConfig,
}

impl InteractCheck {
    pub fn new(config: InteractConfig) -> Self {
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
            findings.extend(self.check_interact(state, place.x, place.y, place.z, place.face, timestamp_ms));
        }

        findings
    }

    fn check_interact(
        &self,
        state: &mut PlayerState,
        block_x: i32,
        block_y: i32,
        block_z: i32,
        face: i32,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Get player's current look direction and position
        let player_yaw = state.movement.last_yaw;
        let player_pitch = state.movement.last_pitch;

        if let Some(ref loc) = state.movement.last_location {
            // Calculate angle to block
            let dx = (block_x as f64 + 0.5) - loc.x;
            let dy = (block_y as f64 + 0.5) - (loc.y + 1.62); // Eye height
            let dz = (block_z as f64 + 0.5) - loc.z;

            let horizontal_dist = (dx * dx + dz * dz).sqrt();
            let expected_yaw = (-dx.atan2(dz)).to_degrees() as f32;
            let expected_pitch = (-dy.atan2(horizontal_dist)).to_degrees() as f32;

            // Normalize yaw difference
            let yaw_diff = normalize_angle(player_yaw - expected_yaw).abs();
            let pitch_diff = (player_pitch - expected_pitch).abs();

            // Check if angle deviation is too large
            if yaw_diff > self.config.max_angle_deviation || pitch_diff > self.config.max_angle_deviation {
                let flagged = state.interact.buffer_angle.fail();
                if flagged {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::InteractAngle,
                            yaw_diff.max(pitch_diff) as f64,
                            state.interact.buffer_angle.vl(),
                            state.interact.buffer_angle.max_vl(),
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "Invalid interact angle: yaw diff {:.1}°, pitch diff {:.1}° (max: {:.0}°)",
                            yaw_diff, pitch_diff, self.config.max_angle_deviation
                        ))
                        .with_evidence(serde_json::json!({
                            "player_yaw": player_yaw,
                            "player_pitch": player_pitch,
                            "expected_yaw": expected_yaw,
                            "expected_pitch": expected_pitch,
                            "block": [block_x, block_y, block_z],
                            "face": face
                        })),
                    );
                }
            } else {
                state.interact.buffer_angle.pass();
            }

            // Check for impossible angles (interacting through walls, behind player, etc.)
            if self.config.check_impossible_angles {
                // If yaw difference is more than 90 degrees, they're looking away
                if yaw_diff > 90.0 {
                    let flagged = state.interact.buffer_impossible.fail();
                    if flagged {
                        findings.push(
                            Finding::new(
                                state.player_uuid,
                                FeatureId::InteractImpossible,
                                yaw_diff as f64,
                                state.interact.buffer_impossible.vl(),
                                state.interact.buffer_impossible.max_vl(),
                                timestamp_ms,
                            )
                            .with_description(format!(
                                "Impossible interact: looking {:.1}° away from block",
                                yaw_diff
                            ))
                            .with_mitigate(true),
                        );
                    }
                } else {
                    state.interact.buffer_impossible.pass();
                }
            }
        }

        state.interact.last_interact_yaw = player_yaw;
        state.interact.last_interact_pitch = player_pitch;

        findings
    }
}

/// Normalize angle to -180 to 180 range
fn normalize_angle(angle: f32) -> f32 {
    let mut a = angle % 360.0;
    if a > 180.0 {
        a -= 360.0;
    } else if a < -180.0 {
        a += 360.0;
    }
    a
}
