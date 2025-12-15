//! KillAura detection
//!
//! Detects:
//! - Multi-target attacks (switching targets too fast)
//! - Post-attack timing (attacking multiple times too quickly)

use crate::config::KillAuraConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

pub struct KillAuraCheck {
    config: KillAuraConfig,
}

impl KillAuraCheck {
    pub fn new(config: KillAuraConfig) -> Self {
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
                findings.extend(self.check_attack(state, use_entity.entity_id, timestamp_ms));
            }
        }

        findings
    }

    fn check_attack(
        &self,
        state: &mut PlayerState,
        target_id: i32,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        if state.killaura.last_attack_ms > 0 {
            let attack_delay = timestamp_ms - state.killaura.last_attack_ms;

            // Multi-target check: switched targets very quickly
            if let Some(last_target) = state.killaura.last_target_id {
                if last_target != target_id && attack_delay < self.config.multi_target_min_ms {
                    state.killaura.target_switches += 1;
                    let flagged = state.killaura.buffer.fail();
                    
                    if flagged {
                        findings.push(
                            Finding::new(
                                state.player_uuid,
                                FeatureId::KillAuraMultiTarget,
                                attack_delay as f64,
                                state.killaura.buffer.vl(),
                                state.killaura.buffer.max_vl(),
                                timestamp_ms,
                            )
                            .with_description(format!(
                                "Multi-aura: switched targets in {}ms (switches: {})",
                                attack_delay, state.killaura.target_switches
                            )),
                        );
                    }
                } else if last_target == target_id {
                    state.killaura.buffer.pass();
                }
            }

            // Post check: attacking too quickly in general
            if attack_delay < self.config.post_threshold_ms {
                state.killaura.rapid_attacks += 1;
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::KillAuraPost,
                        attack_delay as f64,
                        state.killaura.rapid_attacks,
                        10,
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Post attack: {}ms delay (rapid: {})",
                        attack_delay, state.killaura.rapid_attacks
                    )),
                );
            } else if attack_delay > 100 {
                // Reset rapid attack count after reasonable delay
                state.killaura.rapid_attacks = 0;
            }
        }

        state.killaura.last_attack_ms = timestamp_ms;
        state.killaura.last_target_id = Some(target_id);

        findings
    }
}
