//! Timer detection
//!
//! Detects:
//! - Fast timer (sending packets faster than 20 TPS)
//! - Slow timer (sending packets slower than normal)

use crate::config::TimerConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

pub struct TimerCheck {
    config: TimerConfig,
}

impl TimerCheck {
    pub fn new(config: TimerConfig) -> Self {
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

        // Only check on movement packets (these are sent every tick)
        let is_movement_packet = matches!(
            packet,
            ParsedPacket::Position(_) |
            ParsedPacket::PositionLook(_) |
            ParsedPacket::Look(_) |
            ParsedPacket::Flying(_)
        );

        if !is_movement_packet {
            return Vec::new();
        }

        let mut findings = Vec::new();

        // Skip if recently teleported
        if state.movement.ticks_since_teleport < 20 {
            state.timer.last_packet_ms = timestamp_ms;
            return findings;
        }

        // Initialize window
        if state.timer.window_start_ms == 0 {
            state.timer.window_start_ms = timestamp_ms;
            state.timer.last_packet_ms = timestamp_ms;
            return findings;
        }

        // Track packet timestamps
        state.timer.packet_timestamps.push_back(timestamp_ms);
        state.timer.packets_in_window += 1;

        // Calculate interval since last packet
        let interval = timestamp_ms - state.timer.last_packet_ms;
        state.timer.last_packet_ms = timestamp_ms;

        // Balance tracking: positive = too fast, negative = too slow
        // Each packet should come 50ms apart at 20 TPS
        state.timer.balance_ms += self.config.expected_tick_ms - interval as f64;

        // Clean old timestamps
        let window_start = timestamp_ms - self.config.window_ms;
        while let Some(&front) = state.timer.packet_timestamps.front() {
            if front < window_start {
                state.timer.packet_timestamps.pop_front();
                state.timer.packets_in_window = state.timer.packets_in_window.saturating_sub(1);
            } else {
                break;
            }
        }

        // Reset window if needed
        let window_elapsed = timestamp_ms - state.timer.window_start_ms;
        if window_elapsed >= self.config.window_ms {
            // Check packet rate in window
            if state.timer.packets_in_window >= self.config.min_samples as u32 {
                findings.extend(self.check_timer_rate(state, timestamp_ms));
            }
            
            // Reset window
            state.timer.window_start_ms = timestamp_ms;
            state.timer.packets_in_window = 0;
        }

        // Check balance-based timer
        findings.extend(self.check_timer_balance(state, timestamp_ms));

        findings
    }

    /// Check timer based on packet rate in window
    fn check_timer_rate(
        &self,
        state: &mut PlayerState,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        let expected_packets = (self.config.window_ms as f64 / self.config.expected_tick_ms) as u32;
        let actual_packets = state.timer.packets_in_window;
        
        let deviation_percent = ((actual_packets as f64 - expected_packets as f64) / expected_packets as f64) * 100.0;

        if deviation_percent > self.config.max_deviation_percent {
            // Fast timer
            let flagged = state.timer.buffer_fast.fail_with(deviation_percent / 10.0);
            
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::TimerFast,
                        deviation_percent,
                        state.timer.buffer_fast.vl(),
                        state.timer.buffer_fast.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Fast timer: {:.1}% deviation ({} packets in {}ms, expected {})",
                        deviation_percent, actual_packets, self.config.window_ms, expected_packets
                    ))
                    .with_evidence(serde_json::json!({
                        "deviation_percent": deviation_percent,
                        "actual_packets": actual_packets,
                        "expected_packets": expected_packets,
                        "window_ms": self.config.window_ms
                    })),
                );
            }
        } else if deviation_percent < -self.config.max_deviation_percent {
            // Slow timer
            let flagged = state.timer.buffer_slow.fail_with(deviation_percent.abs() / 10.0);
            
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::TimerSlow,
                        deviation_percent.abs(),
                        state.timer.buffer_slow.vl(),
                        state.timer.buffer_slow.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Slow timer: {:.1}% deviation ({} packets in {}ms, expected {})",
                        deviation_percent.abs(), actual_packets, self.config.window_ms, expected_packets
                    ))
                    .with_evidence(serde_json::json!({
                        "deviation_percent": deviation_percent,
                        "actual_packets": actual_packets,
                        "expected_packets": expected_packets,
                        "window_ms": self.config.window_ms
                    })),
                );
            }
        } else {
            state.timer.buffer_fast.pass();
            state.timer.buffer_slow.pass();
        }

        findings
    }

    /// Check timer based on balance tracking
    fn check_timer_balance(
        &self,
        state: &mut PlayerState,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Balance threshold (250ms = 5 ticks ahead/behind)
        let balance_threshold = 250.0;

        if state.timer.balance_ms > balance_threshold {
            // Too fast
            let flagged = state.timer.buffer_fast.fail();
            
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::TimerFast,
                        state.timer.balance_ms,
                        state.timer.buffer_fast.vl(),
                        state.timer.buffer_fast.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Fast timer (balance): {:.1}ms ahead",
                        state.timer.balance_ms
                    ))
                    .with_evidence(serde_json::json!({
                        "balance_ms": state.timer.balance_ms
                    })),
                );
            }
            
            // Reset balance after flag
            state.timer.balance_ms = 0.0;
        } else if state.timer.balance_ms < -balance_threshold {
            // Too slow
            let flagged = state.timer.buffer_slow.fail();
            
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::TimerSlow,
                        state.timer.balance_ms.abs(),
                        state.timer.buffer_slow.vl(),
                        state.timer.buffer_slow.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Slow timer (balance): {:.1}ms behind",
                        state.timer.balance_ms.abs()
                    ))
                    .with_evidence(serde_json::json!({
                        "balance_ms": state.timer.balance_ms
                    })),
                );
            }
            
            // Reset balance after flag
            state.timer.balance_ms = 0.0;
        }

        // Decay balance towards 0
        state.timer.balance_ms *= 0.99;

        findings
    }
}
