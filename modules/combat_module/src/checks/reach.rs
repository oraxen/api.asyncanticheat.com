//! Reach detection
//!
//! Detects:
//! - Attack distance exceeding vanilla limits
//! - Critical reach violations (definitely cheating)
//!
//! Note: Full reach calculation requires tracking both player and target positions.
//! This implementation uses hitbox offset data when available.

use crate::config::ReachConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

pub struct ReachCheck {
    config: ReachConfig,
}

impl ReachCheck {
    pub fn new(config: ReachConfig) -> Self {
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

        if let ParsedPacket::UseEntity(use_entity) = packet {
            if use_entity.action == "ATTACK" {
                findings.extend(self.check_attack(state, use_entity, timestamp_ms));
            }
        }

        findings
    }

    fn check_attack(
        &self,
        state: &mut PlayerState,
        use_entity: &crate::packets::UseEntityPacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Check if we have target coordinate data (INTERACT_AT data)
        // NOTE: These are relative offsets from entity origin, not absolute positions
        // For full reach calculation, we need both player and entity positions
        if let (Some(tx), Some(ty), Some(tz)) = (
            use_entity.target_x,
            use_entity.target_y,
            use_entity.target_z,
        ) {
            // This is the click offset relative to entity origin
            // Very large offsets indicate invalid click positions
            let hitbox_offset = (tx * tx + ty * ty + tz * tz).sqrt();

            // Hitbox offsets > 2.0 are suspicious (normal entity is ~0.6x1.8)
            if hitbox_offset > 2.0 {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::ReachDistance,
                        hitbox_offset,
                        1,
                        5,
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Invalid hitbox offset: {:.2} (expected < 2.0)",
                        hitbox_offset
                    )),
                );
            }
        }

        // For actual reach distance (player to entity), we track via reach_samples
        // This requires distance calculation in the API transform
        if state.reach.last_reach > self.config.max_reach {
            let violation = state.reach.last_reach - self.config.max_reach;
            state.reach.vl += violation;

            if state.reach.last_reach > self.config.critical_reach {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::ReachCritical,
                        state.reach.last_reach,
                        state.reach.buffer.vl(),
                        state.reach.buffer.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Critical reach: {:.2} blocks (max: {:.1})",
                        state.reach.last_reach, self.config.critical_reach
                    ))
                    .with_mitigate(true),
                );
            } else {
                let flagged = state.reach.buffer.fail_with(violation);
                if flagged {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::ReachDistance,
                            state.reach.last_reach,
                            state.reach.buffer.vl(),
                            state.reach.buffer.max_vl(),
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "Reach: {:.2} blocks (max: {:.1}, vl: {:.1})",
                            state.reach.last_reach, self.config.max_reach, state.reach.vl
                        )),
                    );
                }
            }
        } else {
            state.reach.buffer.pass();
            state.reach.vl *= 0.8; // Decay VL
        }

        findings
    }

    /// Set reach distance from external calculation (e.g., API transform)
    pub fn set_reach_distance(state: &mut PlayerState, distance: f64) {
        state.reach.last_reach = distance;
        state.reach.reach_samples.push(distance);
    }
}
