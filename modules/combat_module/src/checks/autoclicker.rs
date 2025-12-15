//! AutoClicker detection
//!
//! Detects:
//! - CPS limits (clicks per second)
//! - Timing patterns (low variance, low std dev)
//! - Statistical anomalies (kurtosis)
//! - Tick alignment (clicks aligned to 50ms ticks)

use crate::config::AutoClickerConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

/// CPS check window (ms)
const CPS_WINDOW_MS: i64 = 1000;
/// Tick duration in ms
const TICK_MS: i64 = 50;

pub struct AutoClickerCheck {
    config: AutoClickerConfig,
}

impl AutoClickerCheck {
    pub fn new(config: AutoClickerConfig) -> Self {
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
                findings.extend(self.check_attack(state, timestamp_ms));
            }
        }

        findings
    }

    fn check_attack(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Record click interval
        if state.autoclicker.last_click_ms > 0 {
            let interval = timestamp_ms - state.autoclicker.last_click_ms;
            if interval > 0 {
                state.autoclicker.click_intervals.push(interval as f64);

                // Tick alignment check
                if self.config.check_tick_alignment {
                    findings.extend(self.check_tick_alignment(state, interval, timestamp_ms));
                }

                // Statistical analysis when buffer is full
                if state.autoclicker.click_intervals.is_full() {
                    findings.extend(self.check_statistics(state, timestamp_ms));
                }
            }
        }

        // CPS tracking
        if state.autoclicker.window_start_ms == 0 {
            state.autoclicker.window_start_ms = timestamp_ms;
            state.autoclicker.clicks_in_window = 0;
        }

        state.autoclicker.clicks_in_window += 1;

        let window_elapsed = timestamp_ms - state.autoclicker.window_start_ms;
        if window_elapsed >= CPS_WINDOW_MS {
            findings.extend(self.check_cps(state, window_elapsed, timestamp_ms));
            state.autoclicker.clicks_in_window = 0;
            state.autoclicker.window_start_ms = timestamp_ms;
        }

        state.autoclicker.last_click_ms = timestamp_ms;
        findings
    }

    fn check_cps(&self, state: &mut PlayerState, window_elapsed: i64, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        let cps = (state.autoclicker.clicks_in_window as f64 * 1000.0) / window_elapsed as f64;
        state.autoclicker.last_stats.cps = cps;

        if cps > self.config.max_cps {
            let flagged = state.autoclicker.buffer_cps.fail();
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AutoClickerCps,
                        cps,
                        state.autoclicker.buffer_cps.vl(),
                        state.autoclicker.buffer_cps.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!("Impossible CPS: {:.1} (max: {:.0})", cps, self.config.max_cps))
                    .with_mitigate(true),
                );
            }
        } else if cps > self.config.suspicious_cps {
            let flagged = state.autoclicker.buffer_cps.fail_with(0.5);
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AutoClickerCps,
                        cps,
                        state.autoclicker.buffer_cps.vl(),
                        state.autoclicker.buffer_cps.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!("High CPS: {:.1}", cps)),
                );
            }
        } else {
            state.autoclicker.buffer_cps.pass();
        }

        findings
    }

    fn check_statistics(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        let std_dev = state.autoclicker.click_intervals.std_dev();
        let variance = state.autoclicker.click_intervals.variance();
        let kurtosis = state.autoclicker.click_intervals.kurtosis();
        let mean = state.autoclicker.click_intervals.mean();

        state.autoclicker.last_stats.std_dev = std_dev;
        state.autoclicker.last_stats.variance = variance;
        state.autoclicker.last_stats.kurtosis = kurtosis;
        state.autoclicker.last_stats.distinct = state.autoclicker.click_intervals.distinct_count();

        // Low standard deviation check
        if std_dev < self.config.low_std_dev_threshold && state.autoclicker.last_stats.cps > 5.0 {
            let flagged = state.autoclicker.buffer_timing.fail();
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AutoClickerTiming,
                        std_dev,
                        state.autoclicker.buffer_timing.vl(),
                        state.autoclicker.buffer_timing.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Low std dev: {:.1}ms (mean: {:.1}ms)",
                        std_dev, mean
                    )),
                );
            }
        } else {
            state.autoclicker.buffer_timing.pass();
        }

        // Low variance check
        if variance < self.config.low_variance_threshold && state.autoclicker.last_stats.cps > 8.0 {
            let flagged = state.autoclicker.buffer_variance.fail();
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AutoClickerVariance,
                        variance,
                        state.autoclicker.buffer_variance.vl(),
                        state.autoclicker.buffer_variance.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!("Low variance: {:.1}", variance)),
                );
            }
        } else {
            state.autoclicker.buffer_variance.pass();
        }

        // Low kurtosis check (indicates uniform distribution)
        if kurtosis < -1.0 && state.autoclicker.last_stats.cps > 8.0 {
            let flagged = state.autoclicker.buffer_kurtosis.fail();
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AutoClickerKurtosis,
                        kurtosis,
                        state.autoclicker.buffer_kurtosis.vl(),
                        state.autoclicker.buffer_kurtosis.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!("Low kurtosis: {:.2}", kurtosis)),
                );
            }
        } else {
            state.autoclicker.buffer_kurtosis.pass();
        }

        findings
    }

    fn check_tick_alignment(&self, state: &mut PlayerState, interval: i64, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        let remainder = interval % TICK_MS;
        let is_tick_aligned = remainder < 5 || remainder > TICK_MS - 5;

        if is_tick_aligned && state.autoclicker.click_intervals.len() >= 5 {
            // Count aligned clicks
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

            if alignment_ratio > 0.8 {
                let flagged = state.autoclicker.buffer_tickalign.fail();
                if flagged {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::AutoClickerTickAlign,
                            alignment_ratio as f64,
                            state.autoclicker.buffer_tickalign.vl(),
                            state.autoclicker.buffer_tickalign.max_vl(),
                            timestamp_ms,
                        )
                        .with_description(format!("{:.0}% tick-aligned", alignment_ratio * 100.0)),
                    );
                }
            } else {
                state.autoclicker.buffer_tickalign.pass();
            }
        }

        findings
    }
}
