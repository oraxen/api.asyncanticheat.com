//! Speed detection
//!
//! Detects:
//! - Horizontal speed exceeding limits
//! - Moving too fast while sprinting
//! - Moving too fast while sneaking

use crate::config::SpeedConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

pub struct SpeedCheck {
    config: SpeedConfig,
}

impl SpeedCheck {
    pub fn new(config: SpeedConfig) -> Self {
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

        // Only check on position packets
        let (x, z, on_ground) = match packet {
            ParsedPacket::Position(p) => (p.x, p.z, p.on_ground),
            ParsedPacket::PositionLook(p) => (p.x, p.z, p.on_ground),
            _ => return findings,
        };

        // Skip if recently teleported
        if state.movement.ticks_since_teleport < 5 {
            return findings;
        }

        // Need previous location
        let prev = match state.movement.last_location {
            Some(loc) => loc,
            None => return findings,
        };

        // Calculate horizontal speed
        let dx = x - prev.x;
        let dz = z - prev.z;
        let horizontal_speed = (dx * dx + dz * dz).sqrt();

        // Track speed samples
        state.speed.speed_samples.push(horizontal_speed);

        // Skip small movements (standing still)
        if horizontal_speed < 0.01 {
            state.speed.buffer_horizontal.pass();
            state.speed.buffer_sprint.pass();
            state.speed.buffer_sneak.pass();
            return findings;
        }

        // Determine max allowed speed based on state
        let (max_speed, check_type) = if state.speed.is_sneaking {
            (self.config.max_sneak_speed, SpeedCheckType::Sneak)
        } else if state.speed.is_sprinting {
            (self.config.max_sprint_speed, SpeedCheckType::Sprint)
        } else {
            (self.config.max_walk_speed, SpeedCheckType::Walk)
        };

        let max_allowed = max_speed * self.config.tolerance;

        // Check speed violation
        if horizontal_speed > max_allowed {
            let excess = horizontal_speed - max_allowed;
            
            findings.extend(self.flag_speed(
                state,
                horizontal_speed,
                max_allowed,
                excess,
                check_type,
                on_ground,
                timestamp_ms,
            ));
        } else {
            // Pass on all buffers
            state.speed.buffer_horizontal.pass();
            state.speed.buffer_sprint.pass();
            state.speed.buffer_sneak.pass();
        }

        findings
    }

    fn flag_speed(
        &self,
        state: &mut PlayerState,
        speed: f64,
        max_allowed: f64,
        excess: f64,
        check_type: SpeedCheckType,
        on_ground: bool,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        let (feature_id, buffer) = match check_type {
            SpeedCheckType::Walk => {
                (FeatureId::SpeedHorizontal, &mut state.speed.buffer_horizontal)
            }
            SpeedCheckType::Sprint => {
                (FeatureId::SpeedSprint, &mut state.speed.buffer_sprint)
            }
            SpeedCheckType::Sneak => {
                (FeatureId::SpeedSneak, &mut state.speed.buffer_sneak)
            }
        };

        let flagged = buffer.fail_with(excess);

        if flagged {
            findings.push(
                Finding::new(
                    state.player_uuid,
                    feature_id,
                    speed,
                    buffer.vl(),
                    buffer.max_vl(),
                    timestamp_ms,
                )
                .with_description(format!(
                    "Speed violation: {:.4} > {:.4} (excess: {:.4}, type: {:?})",
                    speed, max_allowed, excess, check_type
                ))
                .with_evidence(serde_json::json!({
                    "speed": speed,
                    "max_allowed": max_allowed,
                    "excess": excess,
                    "check_type": format!("{:?}", check_type),
                    "on_ground": on_ground,
                    "sprinting": state.speed.is_sprinting,
                    "sneaking": state.speed.is_sneaking
                })),
            );
        }

        findings
    }
}

#[derive(Debug, Clone, Copy)]
enum SpeedCheckType {
    Walk,
    Sprint,
    Sneak,
}
