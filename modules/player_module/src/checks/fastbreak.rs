//! FastBreak detection
//!
//! Detects:
//! - Breaking blocks faster than humanly possible
//! - Critical fast breaking (instant break)

use crate::config::FastBreakConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

pub struct FastBreakCheck {
    config: FastBreakConfig,
}

impl FastBreakCheck {
    pub fn new(config: FastBreakConfig) -> Self {
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

        if let ParsedPacket::BlockDig(dig) = packet {
            // Only check on completed breaks
            if dig.status == "FINISHED" || dig.status == "DONE" || dig.status == "STOP" {
                findings.extend(self.check_break(state, timestamp_ms));
            }
        }

        findings
    }

    fn check_break(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        if state.fastbreak.last_break_ms > 0 {
            let interval = timestamp_ms - state.fastbreak.last_break_ms;

            if interval > 0 {
                state.fastbreak.break_intervals.push(interval as f64);

                // Critical check - impossibly fast (instant break)
                if interval < self.config.critical_interval_ms {
                    let flagged = state.fastbreak.buffer_critical.fail();
                    if flagged {
                        findings.push(
                            Finding::new(
                                state.player_uuid,
                                FeatureId::FastBreakCritical,
                                interval as f64,
                                state.fastbreak.buffer_critical.vl(),
                                state.fastbreak.buffer_critical.max_vl(),
                                timestamp_ms,
                            )
                            .with_description(format!(
                                "Critical fast break: {}ms interval (min: {}ms)",
                                interval, self.config.critical_interval_ms
                            ))
                            .with_mitigate(true),
                        );
                    }
                    state.fastbreak.fast_break_count += 1;
                }
                // Standard fast break check
                else if interval < self.config.min_break_interval_ms {
                    let flagged = state.fastbreak.buffer.fail();
                    if flagged {
                        findings.push(
                            Finding::new(
                                state.player_uuid,
                                FeatureId::FastBreak,
                                interval as f64,
                                state.fastbreak.buffer.vl(),
                                state.fastbreak.buffer.max_vl(),
                                timestamp_ms,
                            )
                            .with_description(format!(
                                "Fast break: {}ms interval (min: {}ms)",
                                interval, self.config.min_break_interval_ms
                            )),
                        );
                    }
                    state.fastbreak.fast_break_count += 1;
                } else {
                    state.fastbreak.buffer.pass();
                    state.fastbreak.buffer_critical.pass();
                    // Decay fast break count
                    if state.fastbreak.fast_break_count > 0 {
                        state.fastbreak.fast_break_count = state.fastbreak.fast_break_count.saturating_sub(1);
                    }
                }
            }
        }

        state.fastbreak.last_break_ms = timestamp_ms;
        findings
    }
}
