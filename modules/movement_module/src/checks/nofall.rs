//! NoFall detection
//!
//! Detects:
//! - Invalid ground claims while falling
//! - Fake fall damage avoidance

use crate::config::NoFallConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

pub struct NoFallCheck {
    config: NoFallConfig,
}

impl NoFallCheck {
    pub fn new(config: NoFallConfig) -> Self {
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
        let (y, on_ground) = match packet {
            ParsedPacket::Position(p) => (p.y, p.on_ground),
            ParsedPacket::PositionLook(p) => (p.y, p.on_ground),
            ParsedPacket::Flying(p) => {
                // Update ground state from flying packet
                if let Some(loc) = state.movement.current_location {
                    (loc.y, p.on_ground)
                } else {
                    return findings;
                }
            }
            _ => return findings,
        };

        // Skip if recently teleported
        if state.movement.ticks_since_teleport < 5 {
            state.nofall.fall_distance = 0.0;
            return findings;
        }

        // Get previous Y
        let prev_y = match state.movement.last_location {
            Some(loc) => loc.y,
            None => {
                state.nofall.fall_distance = 0.0;
                return findings;
            }
        };

        let y_delta = y - prev_y;
        state.nofall.last_y_velocity = y_delta;

        // Track fall distance
        if y_delta < 0.0 && !state.movement.was_on_ground {
            state.nofall.fall_distance += y_delta.abs();
        }

        // Check for invalid ground claim
        if on_ground && !state.movement.was_on_ground {
            // Claiming ground after a fall
            if state.nofall.fall_distance >= self.config.min_fall_distance {
                // Check if velocity is too high for a legitimate ground claim
                if y_delta < self.config.ground_claim_max_velocity {
                    state.nofall.invalid_ground_claims += 1;
                    
                    if state.nofall.invalid_ground_claims >= self.config.consecutive_threshold {
                        let flagged = state.nofall.buffer.fail();
                        
                        if flagged {
                            findings.push(
                                Finding::new(
                                    state.player_uuid,
                                    FeatureId::NoFallInvalidGround,
                                    state.nofall.fall_distance,
                                    state.nofall.buffer.vl(),
                                    state.nofall.buffer.max_vl(),
                                    timestamp_ms,
                                )
                                .with_description(format!(
                                    "NoFall: claimed ground after {:.2} blocks fall with velocity {:.4}",
                                    state.nofall.fall_distance, y_delta
                                ))
                                .with_evidence(serde_json::json!({
                                    "fall_distance": state.nofall.fall_distance,
                                    "y_velocity": y_delta,
                                    "invalid_claims": state.nofall.invalid_ground_claims
                                })),
                            );
                        }
                    }
                } else {
                    // Valid ground claim
                    state.nofall.invalid_ground_claims = 0;
                    state.nofall.buffer.pass();
                }
            }
            
            // Reset fall tracking
            state.nofall.fall_distance = 0.0;
        }

        // Reset when legitimately on ground
        if on_ground && y_delta.abs() < 0.01 {
            state.nofall.fall_distance = 0.0;
            state.nofall.invalid_ground_claims = 0;
        }

        findings
    }
}
