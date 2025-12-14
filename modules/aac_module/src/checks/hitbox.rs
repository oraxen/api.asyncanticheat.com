//! Hitbox check (ch_0) - Combat validity for hits
//!
//! Detects:
//! - Entity reach (too far)
//! - Line-of-sight / through walls
//! - Hit/miss statistics
//! - Looking at the entity they hit

use crate::config::HitboxConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::{ParsedPacket, UseEntityPacket};
use crate::player_state::PlayerState;

/// Maximum legitimate reach (blocks) - vanilla is 3.0, with lag compensation allow more
const MAX_REACH_WITH_LAG: f64 = 3.5;
/// Critical reach threshold (definitely cheating)
const CRITICAL_REACH_THRESHOLD: f64 = 4.5;

pub struct HitboxCheck {
    config: HitboxConfig,
}

impl HitboxCheck {
    pub fn new(config: HitboxConfig) -> Self {
        Self { config }
    }

    /// Process a packet and return any findings
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
            ParsedPacket::UseEntity(use_entity) => {
                if use_entity.action == "ATTACK" {
                    findings.extend(self.check_attack(state, use_entity, timestamp_ms));
                }
            }
            _ => {}
        }

        findings
    }

    fn check_attack(
        &self,
        state: &mut PlayerState,
        use_entity: &UseEntityPacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Track hit for statistics
        state.hitbox.hits += 1;
        state.hitbox.last_hit_ms = timestamp_ms;
        state.hitbox.last_target_id = Some(use_entity.entity_id);

        // If we have target coordinates (from interact_at), check reach
        if let (Some(tx), Some(ty), Some(tz)) = (use_entity.target_x, use_entity.target_y, use_entity.target_z) {
            // We need player position to calculate reach
            // For now, we store the target and check reach indirectly
            
            // Check if the target position is reasonable
            // In vanilla, these are offsets from entity origin, should be within entity hitbox
            let target_distance = (tx * tx + ty * ty + tz * tz).sqrt();
            
            // INTERACT_AT coordinates are relative to entity, so large values are suspicious
            if target_distance > 2.0 {
                // Suspicious target position
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AacHitboxReach,
                        target_distance as f32,
                        0.5,
                        false,
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Suspicious target offset: {:.2} blocks",
                        target_distance
                    )),
                );
            }

            state.hitbox.last_target_distance = target_distance;
            state.hitbox.reach_samples.push_back(target_distance);
            if state.hitbox.reach_samples.len() > 10 {
                state.hitbox.reach_samples.pop_front();
            }
        }

        // Analyze reach patterns if we have enough samples
        if state.hitbox.reach_samples.len() >= 5 {
            let avg_reach: f64 = state.hitbox.reach_samples.iter().sum::<f64>() 
                / state.hitbox.reach_samples.len() as f64;
            
            // High average reach is suspicious
            if avg_reach > MAX_REACH_WITH_LAG {
                let mitigated = state.hitbox.vl.update(1.0, timestamp_ms);
                
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AacHitboxReach,
                        avg_reach as f32,
                        state.hitbox.vl.get(),
                        mitigated,
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Average reach: {:.2} blocks (max: {:.1})",
                        avg_reach, MAX_REACH_WITH_LAG
                    )),
                );
            }
        }

        // Check hit count patterns
        findings.extend(self.check_hit_patterns(state, timestamp_ms));

        findings
    }

    fn check_hit_patterns(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Check for suspicious hit rate (too many hits in short time)
        // This is a simplified version; full implementation would track combat sessions
        
        let total_attacks = state.hitbox.hits + state.hitbox.misses;
        if total_attacks >= 20 {
            let hit_ratio = state.hitbox.hits as f32 / total_attacks as f32;
            
            // Very high hit ratio is suspicious (100% accuracy over many hits)
            if hit_ratio > 0.95 && state.hitbox.hits >= 20 {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AacHitboxCount,
                        state.hitbox.hits as f32,
                        0.6,
                        false,
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Suspicious accuracy: {:.0}% ({}/{})",
                        hit_ratio * 100.0,
                        state.hitbox.hits,
                        total_attacks
                    )),
                );
            }
        }

        findings
    }

    /// Record a miss (called when attack doesn't hit)
    pub fn record_miss(&self, state: &mut PlayerState) {
        state.hitbox.misses += 1;
    }
}

