//! FastPlace detection
//!
//! Detects:
//! - Placing blocks faster than humanly possible
//! - Critical fast placement (instant)

use crate::config::FastPlaceConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

pub struct FastPlaceCheck {
    config: FastPlaceConfig,
}

impl FastPlaceCheck {
    pub fn new(config: FastPlaceConfig) -> Self {
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

        if let ParsedPacket::BlockPlace(_) = packet {
            findings.extend(self.check_place(state, timestamp_ms));
        }

        findings
    }

    fn check_place(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        if state.fastplace.last_place_ms > 0 {
            let interval = timestamp_ms - state.fastplace.last_place_ms;
            
            if interval > 0 {
                state.fastplace.place_intervals.push(interval as f64);

                // Critical check - impossibly fast
                if interval < self.config.critical_interval_ms {
                    let flagged = state.fastplace.buffer_critical.fail();
                    if flagged {
                        findings.push(
                            Finding::new(
                                state.player_uuid,
                                FeatureId::FastPlaceCritical,
                                interval as f64,
                                state.fastplace.buffer_critical.vl(),
                                state.fastplace.buffer_critical.max_vl(),
                                timestamp_ms,
                            )
                            .with_description(format!(
                                "Critical fast place: {}ms interval (min: {}ms)",
                                interval, self.config.critical_interval_ms
                            ))
                            .with_mitigate(true),
                        );
                    }
                    state.fastplace.fast_place_count += 1;
                }
                // Standard fast place check
                else if interval < self.config.min_place_interval_ms {
                    let flagged = state.fastplace.buffer.fail();
                    if flagged {
                        findings.push(
                            Finding::new(
                                state.player_uuid,
                                FeatureId::FastPlace,
                                interval as f64,
                                state.fastplace.buffer.vl(),
                                state.fastplace.buffer.max_vl(),
                                timestamp_ms,
                            )
                            .with_description(format!(
                                "Fast place: {}ms interval (min: {}ms)",
                                interval, self.config.min_place_interval_ms
                            )),
                        );
                    }
                    state.fastplace.fast_place_count += 1;
                } else {
                    state.fastplace.buffer.pass();
                    state.fastplace.buffer_critical.pass();
                    // Decay fast place count
                    if state.fastplace.fast_place_count > 0 {
                        state.fastplace.fast_place_count = state.fastplace.fast_place_count.saturating_sub(1);
                    }
                }
            }
        }

        state.fastplace.last_place_ms = timestamp_ms;
        findings
    }
}
