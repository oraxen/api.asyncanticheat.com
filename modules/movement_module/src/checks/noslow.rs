//! NoSlow detection
//!
//! Detects:
//! - Moving too fast while using items (eating, blocking, etc.)
//! - Moving too fast while sneaking

use crate::config::NoSlowConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

pub struct NoSlowCheck {
    config: NoSlowConfig,
}

impl NoSlowCheck {
    pub fn new(config: NoSlowConfig) -> Self {
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
        let on_ground = match packet {
            ParsedPacket::Position(p) => p.on_ground,
            ParsedPacket::PositionLook(p) => p.on_ground,
            _ => return findings,
        };

        // Skip if not on ground (air speed is different)
        if !on_ground {
            return findings;
        }

        // Skip if recently teleported
        if state.movement.ticks_since_teleport < 5 {
            return findings;
        }

        // Need both locations
        let (current, prev) = match (state.movement.current_location, state.movement.last_location) {
            (Some(c), Some(p)) => (c, p),
            _ => return findings,
        };

        // Calculate horizontal speed
        let horizontal_speed = current.horizontal_speed(&prev);

        // Skip small movements
        if horizontal_speed < 0.01 {
            return findings;
        }

        // Base walk speed
        let base_speed = crate::config::MAX_WALK_SPEED;

        // Check NoSlow while using item
        if state.noslow.is_using_item {
            let max_speed = base_speed * self.config.using_item_multiplier * self.config.tolerance;
            
            if horizontal_speed > max_speed {
                let flagged = state.noslow.buffer_item.fail_with(horizontal_speed - max_speed);
                
                if flagged {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::NoSlowUsingItem,
                            horizontal_speed,
                            state.noslow.buffer_item.vl(),
                            state.noslow.buffer_item.max_vl(),
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "NoSlow (item): speed={:.4}, max={:.4}",
                            horizontal_speed, max_speed
                        ))
                        .with_evidence(serde_json::json!({
                            "horizontal_speed": horizontal_speed,
                            "max_speed": max_speed,
                            "is_using_item": state.noslow.is_using_item,
                            "using_item_multiplier": self.config.using_item_multiplier
                        })),
                    );
                }
            } else {
                state.noslow.buffer_item.pass();
            }
        }

        // Check NoSlow while sneaking
        if state.speed.is_sneaking {
            let max_speed = base_speed * self.config.sneak_multiplier * self.config.tolerance;
            
            if horizontal_speed > max_speed {
                let flagged = state.noslow.buffer_sneak.fail_with(horizontal_speed - max_speed);
                
                if flagged {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::NoSlowSneaking,
                            horizontal_speed,
                            state.noslow.buffer_sneak.vl(),
                            state.noslow.buffer_sneak.max_vl(),
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "NoSlow (sneak): speed={:.4}, max={:.4}",
                            horizontal_speed, max_speed
                        ))
                        .with_evidence(serde_json::json!({
                            "horizontal_speed": horizontal_speed,
                            "max_speed": max_speed,
                            "is_sneaking": state.speed.is_sneaking,
                            "sneak_multiplier": self.config.sneak_multiplier
                        })),
                    );
                }
            } else {
                state.noslow.buffer_sneak.pass();
            }
        }

        // Pass if not in any slow state
        if !state.noslow.is_using_item && !state.speed.is_sneaking {
            state.noslow.buffer_item.pass();
            state.noslow.buffer_sneak.pass();
        }

        findings
    }
}
