//! NoSwing detection
//!
//! Detects:
//! - Attacks without arm animation (swing)
//! - Missing or delayed swing packets

use crate::config::NoSwingConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

pub struct NoSwingCheck {
    config: NoSwingConfig,
}

impl NoSwingCheck {
    pub fn new(config: NoSwingConfig) -> Self {
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

        match packet {
            ParsedPacket::UseEntity(use_entity) if use_entity.action == "ATTACK" => {
                findings.extend(self.check_attack(state, timestamp_ms));
            }
            ParsedPacket::ArmAnimation(_) => {
                self.handle_swing(state, timestamp_ms);
            }
            _ => {}
        }

        findings
    }

    fn check_attack(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Skip if we haven't seen any swing yet (avoid false positives on startup)
        if state.noswing.last_swing_ms == 0 {
            return findings;
        }

        let swing_age = timestamp_ms - state.noswing.last_swing_ms;

        // Out-of-order packet protection
        if swing_age < 0 {
            return findings;
        }

        // Check if attack happened without recent swing
        if swing_age > self.config.max_swing_age_ms {
            state.noswing.attacks_without_swing += 1;
            state.noswing.vl += 1.0;

            if state.noswing.attacks_without_swing >= self.config.threshold {
                let flagged = state.noswing.buffer.fail();
                if flagged {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::NoSwing,
                            state.noswing.attacks_without_swing as f64,
                            state.noswing.buffer.vl(),
                            state.noswing.buffer.max_vl(),
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "{} attacks without swing (vl: {:.1})",
                            state.noswing.attacks_without_swing, state.noswing.vl
                        )),
                    );
                }
            }
        } else {
            // Valid swing before attack
            state.noswing.attacks_without_swing = 0;
            state.noswing.vl *= 0.9; // Decay
            state.noswing.buffer.pass();
        }

        findings
    }

    fn handle_swing(&self, state: &mut PlayerState, timestamp_ms: i64) {
        // Only update if this is a newer packet
        if timestamp_ms >= state.noswing.last_swing_ms {
            state.noswing.last_swing_ms = timestamp_ms;
        }
    }
}
