//! BadPackets detection
//!
//! Detects:
//! - Invalid pitch (>90 degrees)
//! - NaN positions
//! - Abilities spoofing (flying without permission)
//! - Invalid hotbar slots (outside 0-8)
//! - Flying packet flood

use crate::config::BadPacketsConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

/// Maximum hotbar slot index
const MAX_HOTBAR_SLOT: i32 = 8;

pub struct BadPacketsCheck {
    config: BadPacketsConfig,
}

impl BadPacketsCheck {
    pub fn new(config: BadPacketsConfig) -> Self {
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
            ParsedPacket::Look(look) => {
                findings.extend(self.check_pitch(state, look.pitch, timestamp_ms));
            }
            ParsedPacket::PositionLook(pos_look) => {
                findings.extend(self.check_pitch(state, pos_look.pitch, timestamp_ms));
                if self.config.check_nan_position {
                    findings.extend(self.check_nan_position(state, pos_look.x, pos_look.y, pos_look.z, timestamp_ms));
                }
            }
            ParsedPacket::Position(pos) => {
                if self.config.check_nan_position {
                    findings.extend(self.check_nan_position(state, pos.x, pos.y, pos.z, timestamp_ms));
                }
            }
            ParsedPacket::Flying(_) => {
                findings.extend(self.check_flying_flood(state, timestamp_ms));
            }
            ParsedPacket::HeldItemSlot(held) => {
                findings.extend(self.check_slot(state, held.slot, timestamp_ms));
            }
            ParsedPacket::Abilities(abilities) => {
                if self.config.check_abilities_spoof {
                    findings.extend(self.check_abilities(state, abilities, timestamp_ms));
                }
            }
            _ => {}
        }

        findings
    }

    fn check_pitch(&self, state: &mut PlayerState, pitch: f32, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Check for invalid pitch (absolute value > 90)
        if pitch.abs() > self.config.max_pitch {
            let flagged = state.badpackets.buffer_pitch.fail();
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::BadPacketsPitch,
                        pitch as f64,
                        state.badpackets.buffer_pitch.vl(),
                        state.badpackets.buffer_pitch.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!("Invalid pitch: {:.1}° (max: ±{:.0}°)", pitch, self.config.max_pitch))
                    .with_mitigate(true),
                );
            }
        } else {
            state.badpackets.buffer_pitch.pass();
        }

        // Check for NaN pitch
        if pitch.is_nan() {
            let flagged = state.badpackets.buffer_nan.fail();
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::BadPacketsNaN,
                        0.0,
                        state.badpackets.buffer_nan.vl(),
                        state.badpackets.buffer_nan.max_vl(),
                        timestamp_ms,
                    )
                    .with_description("NaN pitch value".to_string())
                    .with_mitigate(true),
                );
            }
        }

        findings
    }

    fn check_nan_position(&self, state: &mut PlayerState, x: f64, y: f64, z: f64, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        if x.is_nan() || y.is_nan() || z.is_nan() || x.is_infinite() || y.is_infinite() || z.is_infinite() {
            let flagged = state.badpackets.buffer_nan.fail();
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::BadPacketsNaN,
                        0.0,
                        state.badpackets.buffer_nan.vl(),
                        state.badpackets.buffer_nan.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!("Invalid position: ({}, {}, {})", x, y, z))
                    .with_mitigate(true),
                );
            }
        }

        findings
    }

    fn check_slot(&self, state: &mut PlayerState, slot: i32, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        if slot < 0 || slot > MAX_HOTBAR_SLOT {
            let flagged = state.badpackets.buffer_slot.fail();
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::BadPacketsSlot,
                        slot as f64,
                        state.badpackets.buffer_slot.vl(),
                        state.badpackets.buffer_slot.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!("Invalid slot: {} (valid: 0-{})", slot, MAX_HOTBAR_SLOT))
                    .with_mitigate(true),
                );
            }
        } else {
            state.badpackets.buffer_slot.pass();
        }

        findings
    }

    fn check_abilities(&self, state: &mut PlayerState, abilities: &crate::packets::AbilitiesPacket, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Check 1: Flying abilities spoofing
        // If player claims to be flying but server hasn't allowed it
        if abilities.is_flying && !state.badpackets.server_allows_flight {
            let flagged = state.badpackets.buffer_abilities.fail();
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::BadPacketsAbilities,
                        1.0,
                        state.badpackets.buffer_abilities.vl(),
                        state.badpackets.buffer_abilities.max_vl(),
                        timestamp_ms,
                    )
                    .with_description("Abilities spoof: flying without permission".to_string())
                    .with_mitigate(true),
                );
            }
        } else {
            state.badpackets.buffer_abilities.pass();
        }

        // Check 2: Instant break abilities spoofing (creative mode)
        // If player claims instant_break but server hasn't allowed it (not in creative mode)
        // Only flag if instant_break is true AND player is not invulnerable (invulnerable often
        // accompanies creative mode legitimately)
        let instant_break = abilities.instant_break.unwrap_or(false);
        let invulnerable = abilities.invulnerable.unwrap_or(false);
        
        if instant_break && !state.badpackets.server_allows_instant_break && !invulnerable {
            let flagged = state.badpackets.buffer_instant_break.fail();
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::BadPacketsInstantBreak,
                        1.0,
                        state.badpackets.buffer_instant_break.vl(),
                        state.badpackets.buffer_instant_break.max_vl(),
                        timestamp_ms,
                    )
                    .with_description("Abilities spoof: instant_break without creative mode".to_string())
                    .with_mitigate(true),
                );
            }
        } else {
            state.badpackets.buffer_instant_break.pass();
        }

        state.badpackets.last_abilities_flying = abilities.is_flying;
        findings
    }

    fn check_flying_flood(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Reset window if needed
        if state.badpackets.flying_window_start_ms == 0 {
            state.badpackets.flying_window_start_ms = timestamp_ms;
            state.badpackets.flying_packets_this_sec = 0;
        }

        state.badpackets.flying_packets_this_sec += 1;

        let window_elapsed = timestamp_ms - state.badpackets.flying_window_start_ms;
        if window_elapsed >= 1000 {
            let packets_per_sec = state.badpackets.flying_packets_this_sec;
            
            if packets_per_sec > self.config.max_flying_packets_per_sec {
                let flagged = state.badpackets.buffer_flying_flood.fail();
                if flagged {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::BadPacketsFlyingFlood,
                            packets_per_sec as f64,
                            state.badpackets.buffer_flying_flood.vl(),
                            state.badpackets.buffer_flying_flood.max_vl(),
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "Flying packet flood: {} packets/sec (max: {})",
                            packets_per_sec, self.config.max_flying_packets_per_sec
                        )),
                    );
                }
            } else {
                state.badpackets.buffer_flying_flood.pass();
            }

            // Reset window
            state.badpackets.flying_packets_this_sec = 0;
            state.badpackets.flying_window_start_ms = timestamp_ms;
        }

        findings
    }
}
