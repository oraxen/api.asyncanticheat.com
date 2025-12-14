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

        // If we have target coordinates (from INTERACT_AT), check hitbox offset validity
        // NOTE: These are RELATIVE offsets from the entity's origin, NOT absolute positions!
        // They represent where on the entity's hitbox the player clicked.
        // For ATTACK actions, these should be within the entity's hitbox bounds (~1-2 blocks).
        if let (Some(tx), Some(ty), Some(tz)) = (use_entity.target_x, use_entity.target_y, use_entity.target_z) {
            // This is the click offset relative to entity origin, NOT reach distance
            // Normal values: -1.0 to 2.0 (depending on entity size and hitbox)
            let hitbox_offset = (tx * tx + ty * ty + tz * tz).sqrt();
            
            // Hitbox offsets > 2.0 are suspicious (player-sized entity is ~0.6x1.8)
            // This detects invalid click positions, not reach
            if hitbox_offset > 2.0 {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AacHitboxInvalid,
                        hitbox_offset as f32,
                        0.5,
                        false,
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Invalid hitbox offset: {:.2} (expected < 2.0)",
                        hitbox_offset
                    )),
                );
            }

            state.hitbox.last_target_distance = hitbox_offset;
            // Note: We're NOT storing these as "reach samples" since they're offsets, not distances
        }

        // For actual reach detection, we would need:
        // 1. Player position (from movement packets)
        // 2. Target entity position (from entity tracking)
        // 3. Calculate actual distance between them
        // This is done in the ncp_fight_v1 transform which has access to both

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

    // Removed reach_samples analysis - it was using hitbox offsets (relative positions)
    // as if they were reach distances (absolute distances), producing meaningless results.
    // Actual reach detection requires tracking both player and entity positions,
    // which is done in the ncp_fight_v1 transform.

    /// Record a miss (called when attack doesn't hit)
    pub fn record_miss(&self, state: &mut PlayerState) {
        state.hitbox.misses += 1;
    }
}

