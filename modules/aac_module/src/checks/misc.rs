//! Misc check (cO + cC) - Miscellaneous checks
//!
//! Detects:
//! - Invalid pitch values
//! - Invalid player abilities
//! - Excessive rotation rates

use crate::config::MiscConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::{AbilitiesPacket, LookPacket, ParsedPacket, PositionLookPacket};
use crate::player_state::PlayerState;

/// Maximum valid pitch
const MAX_PITCH: f32 = 90.0;
/// Minimum valid pitch  
const MIN_PITCH: f32 = -90.0;
/// Maximum rotation rate (degrees per second)
const MAX_ROTATION_RATE: f32 = 20000.0; // Very high but not infinite
/// Short window duration (ms) - 1 second at 20 TPS
const SHORT_WINDOW_MS: i64 = 1000;
/// Medium window duration (ms) - 10 seconds
const MEDIUM_WINDOW_MS: i64 = 10000;
/// Long window duration (ms) - 60 seconds
const LONG_WINDOW_MS: i64 = 60000;

pub struct MiscCheck {
    config: MiscConfig,
}

impl MiscCheck {
    pub fn new(config: MiscConfig) -> Self {
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
            ParsedPacket::Look(look) => {
                if self.config.invalid_pitch {
                    findings.extend(self.check_pitch(state, look.pitch, timestamp_ms));
                }
                if self.config.rotation_rate {
                    findings.extend(self.check_rotation(state, look.yaw, look.pitch, timestamp_ms));
                }
            }
            ParsedPacket::PositionLook(pos) => {
                if self.config.invalid_pitch {
                    findings.extend(self.check_pitch(state, pos.pitch, timestamp_ms));
                }
                if self.config.rotation_rate {
                    findings.extend(self.check_rotation(state, pos.yaw, pos.pitch, timestamp_ms));
                }
            }
            ParsedPacket::Abilities(abilities) => {
                if self.config.player_abilities {
                    findings.extend(self.check_abilities(state, abilities, timestamp_ms));
                }
            }
            _ => {}
        }

        // Update window counters
        self.update_windows(state, timestamp_ms, &mut findings);

        findings
    }

    fn check_pitch(&self, state: &mut PlayerState, pitch: f32, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Check for invalid pitch values
        if pitch > MAX_PITCH || pitch < MIN_PITCH {
            if !state.misc.invalid_pitch_flagged {
                state.misc.invalid_pitch_flagged = true;

                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AacMiscPitch,
                        pitch,
                        1.0,
                        true, // Should mitigate
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Invalid pitch: {:.1}° (valid range: {}° to {}°)",
                        pitch, MIN_PITCH, MAX_PITCH
                    )),
                );
            }
        } else {
            state.misc.invalid_pitch_flagged = false;
        }

        findings
    }

    fn check_rotation(
        &self,
        state: &mut PlayerState,
        yaw: f32,
        pitch: f32,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        if state.misc.last_rotation_ms > 0 {
            let time_delta_ms = timestamp_ms - state.misc.last_rotation_ms;
            
            if time_delta_ms > 0 {
                let delta_yaw = self.normalize_angle(yaw - state.misc.last_yaw);
                let delta_pitch = pitch - state.misc.last_pitch;
                let total_delta = (delta_yaw.powi(2) + delta_pitch.powi(2)).sqrt();

                // Calculate rotation rate (degrees per second)
                let rotation_rate = (total_delta * 1000.0) / time_delta_ms as f32;

                // Check for excessive rotation rate
                if rotation_rate > MAX_ROTATION_RATE {
                    state.misc.short_window_count += 1;
                    state.misc.medium_window_count += 1;
                    state.misc.long_window_count += 1;
                }

                // Flag immediately if rotation rate is impossibly high
                if rotation_rate > MAX_ROTATION_RATE * 10.0 {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::AacMiscRotation,
                            rotation_rate,
                            1.0,
                            true,
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "Impossible rotation: {:.0}°/s in {}ms",
                            rotation_rate, time_delta_ms
                        )),
                    );
                }
            }
        }

        // Update state
        state.misc.last_yaw = yaw;
        state.misc.last_pitch = pitch;
        state.misc.last_rotation_ms = timestamp_ms;

        findings
    }

    fn check_abilities(
        &self,
        state: &mut PlayerState,
        abilities: &AbilitiesPacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Check for suspicious ability claims
        // Note: In a full implementation, we'd compare against server-known player state
        
        // Flying without allow_flying is always suspicious
        if abilities.flying && !abilities.allow_flying {
            if !state.misc.invalid_abilities_flagged {
                state.misc.invalid_abilities_flagged = true;

                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AacMiscAbilities,
                        1.0,
                        1.0,
                        true,
                        timestamp_ms,
                    )
                    .with_description("Flying without permission".to_string()),
                );
            }
        }

        // Instant break without proper game mode
        if abilities.instant_break && !abilities.invulnerable {
            // instant_break is typically only available in creative mode
            // which also grants invulnerability
            findings.push(
                Finding::new(
                    state.player_uuid,
                    FeatureId::AacMiscAbilities,
                    1.0,
                    0.7,
                    false,
                    timestamp_ms,
                )
                .with_description("Suspicious instant_break ability".to_string()),
            );
        }

        findings
    }

    fn update_windows(
        &self,
        state: &mut PlayerState,
        timestamp_ms: i64,
        findings: &mut Vec<Finding>,
    ) {
        if state.misc.short_window_start_ms == 0 {
            state.misc.short_window_start_ms = timestamp_ms;
        }
        if state.misc.medium_window_start_ms == 0 {
            state.misc.medium_window_start_ms = timestamp_ms;
        }
        if state.misc.long_window_start_ms == 0 {
            state.misc.long_window_start_ms = timestamp_ms;
        }

        // Short window (1s bucket)
        let short_elapsed = timestamp_ms - state.misc.short_window_start_ms;
        if short_elapsed >= SHORT_WINDOW_MS {
            if state.misc.short_window_count > 0 {
                let rate = state.misc.short_window_count as f32;
                if rate > 10.0 {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::AacMiscRotation,
                            rate,
                            rate / 20.0,
                            false,
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "High rotation anomalies: {} in 1s",
                            state.misc.short_window_count
                        )),
                    );
                }
            }
            state.misc.short_window_count = 0;
            // Advance start by full windows elapsed to avoid always-true condition
            let windows = (short_elapsed / SHORT_WINDOW_MS).max(1);
            state.misc.short_window_start_ms += windows * SHORT_WINDOW_MS;
        }

        // Medium window (10s bucket) - currently just reset bucket properly
        let medium_elapsed = timestamp_ms - state.misc.medium_window_start_ms;
        if medium_elapsed >= MEDIUM_WINDOW_MS {
            state.misc.medium_window_count = 0;
            let windows = (medium_elapsed / MEDIUM_WINDOW_MS).max(1);
            state.misc.medium_window_start_ms += windows * MEDIUM_WINDOW_MS;
        }

        // Long window (60s bucket)
        let long_elapsed = timestamp_ms - state.misc.long_window_start_ms;
        if long_elapsed >= LONG_WINDOW_MS {
            let rate = (state.misc.long_window_count as f32 * 20.0) / 1200.0;
            if rate > 1.0 {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AacMiscRotation,
                        rate,
                        rate,
                        false,
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Sustained rotation anomalies: {:.2}/s over 60s",
                        rate
                    )),
                );
            }

            state.misc.short_window_count = 0;
            state.misc.medium_window_count = 0;
            state.misc.long_window_count = 0;
            state.misc.short_window_start_ms = timestamp_ms;
            state.misc.medium_window_start_ms = timestamp_ms;
            state.misc.long_window_start_ms = timestamp_ms;
        }
    }

    fn normalize_angle(&self, angle: f32) -> f32 {
        let mut a = angle;
        while a > 180.0 {
            a -= 360.0;
        }
        while a < -180.0 {
            a += 360.0;
        }
        a
    }
}

