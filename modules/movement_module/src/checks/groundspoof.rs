//! Ground spoof detection
//!
//! Detects:
//! - Claiming ground while falling
//! - Claiming ground while ascending

use crate::config::GroundSpoofConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

pub struct GroundSpoofCheck {
    config: GroundSpoofConfig,
}

impl GroundSpoofCheck {
    pub fn new(config: GroundSpoofConfig) -> Self {
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
            _ => return findings,
        };

        // Skip if recently teleported
        if state.movement.ticks_since_teleport < 5 {
            state.groundspoof.last_ground_y = y;
            return findings;
        }

        // Need previous location
        let prev_y = match state.movement.last_location {
            Some(loc) => loc.y,
            None => {
                state.groundspoof.last_ground_y = y;
                return findings;
            }
        };

        let y_velocity = y - prev_y;
        state.groundspoof.y_velocity = y_velocity;

        // Check for ground claim while falling fast
        if on_ground && y_velocity < -self.config.fall_threshold {
            state.groundspoof.consecutive_spoofs += 1;
            
            if state.groundspoof.consecutive_spoofs >= self.config.consecutive_threshold {
                let flagged = state.groundspoof.buffer.fail();
                
                if flagged {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::GroundSpoofFalling,
                            y_velocity.abs(),
                            state.groundspoof.buffer.vl(),
                            state.groundspoof.buffer.max_vl(),
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "Ground spoof: claimed ground while falling at {:.4} b/t",
                            y_velocity
                        ))
                        .with_evidence(serde_json::json!({
                            "y_velocity": y_velocity,
                            "threshold": self.config.fall_threshold,
                            "consecutive": state.groundspoof.consecutive_spoofs
                        })),
                    );
                }
            }
        }
        // Check for ground claim while ascending (jumping but claiming ground)
        else if on_ground && y_velocity > 0.1 && !state.movement.was_on_ground {
            state.groundspoof.consecutive_spoofs += 1;
            
            if state.groundspoof.consecutive_spoofs >= self.config.consecutive_threshold {
                let flagged = state.groundspoof.buffer.fail();
                
                if flagged {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::GroundSpoofAscending,
                            y_velocity,
                            state.groundspoof.buffer.vl(),
                            state.groundspoof.buffer.max_vl(),
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "Ground spoof: claimed ground while ascending at {:.4} b/t",
                            y_velocity
                        ))
                        .with_evidence(serde_json::json!({
                            "y_velocity": y_velocity,
                            "consecutive": state.groundspoof.consecutive_spoofs,
                            "was_on_ground": state.movement.was_on_ground
                        })),
                    );
                }
            }
        }
        // Valid ground state
        else if on_ground {
            state.groundspoof.consecutive_spoofs = 0;
            state.groundspoof.buffer.pass();
            state.groundspoof.last_ground_y = y;
        }
        // In air
        else {
            state.groundspoof.consecutive_spoofs = 0;
        }

        findings
    }
}
