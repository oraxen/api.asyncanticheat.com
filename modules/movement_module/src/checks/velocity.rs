//! Velocity detection
//!
//! Detects:
//! - Ignoring knockback/velocity
//! - Partial velocity application

use crate::config::VelocityConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::{PlayerState, PendingVelocity};

pub struct VelocityCheck {
    config: VelocityConfig,
}

impl VelocityCheck {
    pub fn new(config: VelocityConfig) -> Self {
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

        // Handle velocity packet
        if let ParsedPacket::EntityVelocity(vel) = packet {
            // Only track player's own velocity (entity_id would match player's entity id)
            // In practice, this is sent with entityId = player's entity ID
            let magnitude = (vel.velocity_x.powi(2) + vel.velocity_y.powi(2) + vel.velocity_z.powi(2)).sqrt();
            
            if magnitude >= self.config.min_velocity {
                state.velocity.pending_velocity = Some(PendingVelocity {
                    x: vel.velocity_x,
                    y: vel.velocity_y,
                    z: vel.velocity_z,
                    timestamp_ms,
                    ticks_elapsed: 0,
                });
                state.velocity.last_velocity_ms = timestamp_ms;
            }
            return findings;
        }

        // Check velocity response on movement packets
        let (x, y, z) = match packet {
            ParsedPacket::Position(p) => (p.x, p.z, p.y),
            ParsedPacket::PositionLook(p) => (p.x, p.z, p.y),
            _ => return findings,
        };

        // Get previous location
        let prev = match state.movement.last_location {
            Some(loc) => loc,
            None => return findings,
        };

        // Check pending velocity
        if let Some(ref mut pending) = state.velocity.pending_velocity {
            pending.ticks_elapsed += 1;

            // Calculate movement deltas
            let dx = x - prev.x;
            let dz = z - prev.z;
            let dy = y - prev.y;

            // Check if velocity was applied (movement in similar direction)
            let expected_direction_h = (pending.x.powi(2) + pending.z.powi(2)).sqrt();
            let actual_direction_h = (dx.powi(2) + dz.powi(2)).sqrt();

            // Calculate how much velocity was applied (percentage)
            let horizontal_percent = if expected_direction_h > 0.01 {
                (actual_direction_h / expected_direction_h) * 100.0
            } else {
                100.0
            };

            let vertical_percent = if pending.y.abs() > 0.01 {
                (dy / pending.y) * 100.0
            } else {
                100.0
            };

            // Check if velocity was responded to within tick limit
            if pending.ticks_elapsed >= self.config.max_response_ticks {
                // Timed out - check if velocity was ignored
                if horizontal_percent < self.config.min_velocity_percent 
                    || vertical_percent < self.config.min_velocity_percent 
                {
                    state.velocity.ignored_count += 1;
                    
                    if state.velocity.ignored_count >= self.config.ignore_threshold {
                        let flagged = state.velocity.buffer.fail();
                        
                        if flagged {
                            findings.push(
                                Finding::new(
                                    state.player_uuid,
                                    FeatureId::VelocityIgnored,
                                    horizontal_percent.min(vertical_percent),
                                    state.velocity.buffer.vl(),
                                    state.velocity.buffer.max_vl(),
                                    timestamp_ms,
                                )
                                .with_description(format!(
                                    "Velocity ignored: h={:.1}%, v={:.1}% (min: {:.1}%)",
                                    horizontal_percent, vertical_percent, self.config.min_velocity_percent
                                ))
                                .with_evidence(serde_json::json!({
                                    "horizontal_percent": horizontal_percent,
                                    "vertical_percent": vertical_percent,
                                    "expected_velocity": {
                                        "x": pending.x,
                                        "y": pending.y,
                                        "z": pending.z
                                    },
                                    "actual_delta": {
                                        "x": dx,
                                        "y": dy,
                                        "z": dz
                                    },
                                    "ticks_elapsed": pending.ticks_elapsed,
                                    "ignored_count": state.velocity.ignored_count
                                })),
                            );
                        }
                    }
                } else {
                    // Velocity was eventually applied
                    state.velocity.ignored_count = 0;
                    state.velocity.buffer.pass();
                }
                
                // Clear pending velocity
                state.velocity.pending_velocity = None;
            }
            // Check for partial velocity (applied less than expected)
            else if pending.ticks_elapsed > 2 {
                // After a few ticks, check if velocity is being partially applied
                if horizontal_percent < self.config.min_velocity_percent 
                    && horizontal_percent > 10.0 
                {
                    // Partial application detected
                    let flagged = state.velocity.buffer.fail_with(0.5);
                    
                    if flagged {
                        findings.push(
                            Finding::new(
                                state.player_uuid,
                                FeatureId::VelocityPartial,
                                horizontal_percent,
                                state.velocity.buffer.vl(),
                                state.velocity.buffer.max_vl(),
                                timestamp_ms,
                            )
                            .with_description(format!(
                                "Partial velocity: h={:.1}%, v={:.1}%",
                                horizontal_percent, vertical_percent
                            ))
                            .with_evidence(serde_json::json!({
                                "horizontal_percent": horizontal_percent,
                                "vertical_percent": vertical_percent,
                                "ticks_elapsed": pending.ticks_elapsed
                            })),
                        );
                    }
                }
            }
        }

        findings
    }
}
