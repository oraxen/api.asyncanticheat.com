//! Step detection
//!
//! Detects:
//! - Climbing higher than vanilla step height (0.6 blocks)
//! - Step without being on ground

use crate::config::StepConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

pub struct StepCheck {
    config: StepConfig,
}

impl StepCheck {
    pub fn new(config: StepConfig) -> Self {
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
            state.step.last_y = y;
            state.step.was_on_ground = on_ground;
            return findings;
        }

        let y_delta = y - state.step.last_y;

        // Check for step: ascending while on ground (both before and after)
        if y_delta > 0.0 && state.step.was_on_ground && on_ground {
            // This is a step (went up and stayed on ground)
            
            if y_delta > self.config.max_step_height {
                let flagged = state.step.buffer.fail_with(y_delta - self.config.max_step_height);
                
                if flagged {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::StepHeight,
                            y_delta,
                            state.step.buffer.vl(),
                            state.step.buffer.max_vl(),
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "Step height violation: {:.4} > {:.4}",
                            y_delta, self.config.max_step_height
                        ))
                        .with_evidence(serde_json::json!({
                            "y_delta": y_delta,
                            "max_step": self.config.max_step_height,
                            "was_on_ground": state.step.was_on_ground,
                            "on_ground": on_ground
                        })),
                    );
                }
            } else {
                state.step.buffer.pass();
            }
        }

        // Check for step without ground (stepping while in air)
        if self.config.require_ground && !state.step.was_on_ground && on_ground {
            // Landing after being in air - check if it looks like an invalid step
            let prev_y = state.step.last_y;
            if y_delta > self.config.max_step_height && y_delta < 1.5 {
                // Suspicious: went up significantly while claiming to land on ground
                let flagged = state.step.buffer.fail();
                
                if flagged {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::StepNoGround,
                            y_delta,
                            state.step.buffer.vl(),
                            state.step.buffer.max_vl(),
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "Step without ground: y_delta={:.4} from air to ground",
                            y_delta
                        ))
                        .with_evidence(serde_json::json!({
                            "y_delta": y_delta,
                            "prev_y": prev_y,
                            "current_y": y,
                            "was_on_ground": state.step.was_on_ground,
                            "on_ground": on_ground
                        })),
                    );
                }
            }
        }

        state.step.last_y = y;
        state.step.was_on_ground = on_ground;

        findings
    }
}
