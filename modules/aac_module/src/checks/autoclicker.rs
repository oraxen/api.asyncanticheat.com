//! Autoclicker check (cr_0) - Abnormal click patterns
//!
//! Detects:
//! - Click timing consistency / irregularity
//! - No swing attacks
//! - Tick-aligned click delays

use crate::config::AutoclickerConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::{ArmAnimationPacket, ParsedPacket, UseEntityPacket};
use crate::player_state::PlayerState;

/// CPS check window (ms)
const CPS_WINDOW_MS: i64 = 1000;
/// Maximum legitimate CPS
const MAX_LEGITIMATE_CPS: f32 = 20.0;
/// Suspicious CPS threshold
const SUSPICIOUS_CPS_THRESHOLD: f32 = 16.0;
/// Tick duration in ms (50ms = 1 tick)
const TICK_MS: i64 = 50;
/// Variance threshold for autoclicker detection
const MIN_VARIANCE_THRESHOLD: f64 = 5.0; // ms²

pub struct AutoclickerCheck {
    config: AutoclickerConfig,
}

impl AutoclickerCheck {
    pub fn new(config: AutoclickerConfig) -> Self {
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
            ParsedPacket::ArmAnimation(anim) => {
                self.handle_swing(state, anim, timestamp_ms);
            }
            _ => {}
        }

        findings
    }

    fn check_attack(
        &self,
        state: &mut PlayerState,
        _use_entity: &UseEntityPacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Record click interval
        if state.autoclicker.last_click_ms > 0 {
            let interval = timestamp_ms - state.autoclicker.last_click_ms;
            // Ignore out-of-order timestamps (negative/zero intervals) to avoid polluting timing stats.
            if interval > 0 {
                state.autoclicker.click_intervals.push(interval as f64);

                // Check tick alignment
                if self.config.check_tick_delay {
                    findings.extend(self.check_tick_alignment(state, interval, timestamp_ms));
                }

                // Check timing patterns when buffer is full
                if self.config.check_timing && state.autoclicker.click_intervals.is_full() {
                    findings.extend(self.check_timing_patterns(state, timestamp_ms));
                }
            }
        }

        // Update CPS tracking
        if state.autoclicker.window_start_ms == 0 {
            state.autoclicker.window_start_ms = timestamp_ms;
            state.autoclicker.clicks_in_window = 0;
        }
        // Out-of-order timestamps: do not reset the CPS window (prevents evasion and state corruption).
        if timestamp_ms >= state.autoclicker.window_start_ms {
            state.autoclicker.clicks_in_window += 1;

            // Check CPS at window boundary
            let window_elapsed = timestamp_ms - state.autoclicker.window_start_ms;
            if window_elapsed >= CPS_WINDOW_MS {
                findings.extend(self.check_cps(state, timestamp_ms));

                // Reset window
                state.autoclicker.clicks_in_window = 0;
                state.autoclicker.window_start_ms = timestamp_ms;
            }
        }

        // Check no-swing
        if self.config.check_noswing {
            findings.extend(self.check_noswing(state, timestamp_ms));
        }

        // Only advance last_click_ms if this packet is not older than what we've already seen.
        if timestamp_ms >= state.autoclicker.last_click_ms {
            state.autoclicker.last_click_ms = timestamp_ms;
        }
        findings
    }

    fn check_tick_alignment(
        &self,
        state: &mut PlayerState,
        interval: i64,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Check if interval is suspiciously tick-aligned
        let remainder = interval % TICK_MS;
        let is_tick_aligned = remainder < 5 || remainder > TICK_MS - 5;

        if is_tick_aligned && state.autoclicker.click_intervals.len() >= 5 {
            // Count how many recent intervals are tick-aligned
            let aligned_count: usize = state
                .autoclicker
                .click_intervals
                .iter()
                .filter(|&&i| {
                    let r = (i as i64) % TICK_MS;
                    r < 5 || r > TICK_MS - 5
                })
                .count();

            let alignment_ratio = aligned_count as f32 / state.autoclicker.click_intervals.len() as f32;

            // High tick alignment is suspicious (humans have random variance)
            if alignment_ratio > 0.8 {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AacClickTickDelay,
                        alignment_ratio,
                        alignment_ratio,
                        false,
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "{:.0}% of clicks are tick-aligned",
                        alignment_ratio * 100.0
                    )),
                );
            }
        }

        findings
    }

    fn check_timing_patterns(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        let variance = state.autoclicker.click_intervals.variance();
        let std_dev = variance.sqrt();
        let mean = state.autoclicker.click_intervals.mean();

        // Very low variance indicates autoclicker
        if variance < MIN_VARIANCE_THRESHOLD && mean < 100.0 {
            // High CPS with low variance
            findings.push(
                Finding::new(
                    state.player_uuid,
                    FeatureId::AacClickTiming,
                    std_dev as f32,
                    0.7,
                    false,
                    timestamp_ms,
                )
                .with_description(format!(
                    "Low click variance: {:.2}ms (mean interval: {:.1}ms)",
                    std_dev, mean
                )),
            );
        }

        // Calculate coefficient of variation (CV)
        if mean > 0.0 {
            let cv = std_dev / mean;
            
            // Very consistent clicking (CV < 0.1) is suspicious
            if cv < 0.1 && mean < 150.0 {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AacClickVar,
                        cv as f32,
                        0.6,
                        false,
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Coefficient of variation: {:.3} (suspicious consistency)",
                        cv
                    )),
                );
            }
        }

        findings
    }

    fn check_cps(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        let window_elapsed = timestamp_ms - state.autoclicker.window_start_ms;
        if window_elapsed > 0 {
            let cps = (state.autoclicker.clicks_in_window as f64 * 1000.0) / window_elapsed as f64;

            if cps > MAX_LEGITIMATE_CPS as f64 {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AacClickCps,
                        cps as f32,
                        1.0,
                        true, // Should mitigate
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Impossible CPS: {:.1} (max legitimate: {})",
                        cps, MAX_LEGITIMATE_CPS
                    )),
                );
            } else if cps > SUSPICIOUS_CPS_THRESHOLD as f64 {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AacClickCps,
                        cps as f32,
                        0.7,
                        false,
                        timestamp_ms,
                    )
                    .with_description(format!("High CPS: {:.1}", cps)),
                );
            }
        }

        findings
    }

    fn check_noswing(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Skip check if we haven't seen any swing yet (avoid false positives on startup)
        if state.autoclicker.last_swing_ms == 0 {
            return findings;
        }

        // Check if attack happened without recent swing
        let swing_age = timestamp_ms - state.autoclicker.last_swing_ms;
        // Out-of-order timestamps: don't let negative ages reset counters.
        if swing_age < 0 {
            return findings;
        }
        
        // Swing should come before or very shortly after attack (within 50ms)
        if swing_age > 100 {
            state.autoclicker.attacks_without_swing += 1;

            if state.autoclicker.attacks_without_swing >= 5 {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AacClickNoswing,
                        state.autoclicker.attacks_without_swing as f32,
                        0.8,
                        false,
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "{} attacks without arm swing",
                        state.autoclicker.attacks_without_swing
                    )),
                );
            }
        } else {
            state.autoclicker.attacks_without_swing = 0;
        }

        findings
    }

    fn handle_swing(&self, state: &mut PlayerState, _anim: &ArmAnimationPacket, timestamp_ms: i64) {
        // Out-of-order packets can arrive; don't move swing time backwards.
        if timestamp_ms >= state.autoclicker.last_swing_ms {
        state.autoclicker.last_swing_ms = timestamp_ms;
        }
    }
}

