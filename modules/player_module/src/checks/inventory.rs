//! Inventory detection
//!
//! Detects:
//! - Fast inventory clicks (clicking faster than humanly possible)

use crate::config::InventoryConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

pub struct InventoryCheck {
    config: InventoryConfig,
}

impl InventoryCheck {
    pub fn new(config: InventoryConfig) -> Self {
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

        if let ParsedPacket::WindowClick(click) = packet {
            findings.extend(self.check_click(state, click.window_id, click.slot, timestamp_ms));
        }

        findings
    }

    fn check_click(
        &self,
        state: &mut PlayerState,
        _window_id: i32,
        _slot: i32,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        if state.inventory.last_click_ms > 0 {
            let interval = timestamp_ms - state.inventory.last_click_ms;

            if interval > 0 {
                state.inventory.click_intervals.push(interval as f64);

                // Check for fast clicks
                if interval < self.config.fastclick_window_ms {
                    state.inventory.fast_clicks_count += 1;

                    if state.inventory.fast_clicks_count >= self.config.fast_click_threshold {
                        let flagged = state.inventory.buffer.fail();
                        if flagged {
                            let avg_interval = if state.inventory.click_intervals.len() > 0 {
                                state.inventory.click_intervals.mean()
                            } else {
                                interval as f64
                            };

                            findings.push(
                                Finding::new(
                                    state.player_uuid,
                                    FeatureId::InventoryFastClick,
                                    avg_interval,
                                    state.inventory.buffer.vl(),
                                    state.inventory.buffer.max_vl(),
                                    timestamp_ms,
                                )
                                .with_description(format!(
                                    "Fast inventory clicks: {} clicks under {}ms (last: {}ms, avg: {:.1}ms)",
                                    state.inventory.fast_clicks_count,
                                    self.config.fastclick_window_ms,
                                    interval,
                                    avg_interval
                                ))
                                .with_evidence(serde_json::json!({
                                    "fast_clicks": state.inventory.fast_clicks_count,
                                    "last_interval": interval,
                                    "avg_interval": avg_interval,
                                    "threshold_ms": self.config.fastclick_window_ms
                                })),
                            );
                        }
                    }
                } else {
                    state.inventory.buffer.pass();
                    // Decay fast click count
                    if state.inventory.fast_clicks_count > 0 {
                        state.inventory.fast_clicks_count = state.inventory.fast_clicks_count.saturating_sub(1);
                    }
                }
            }
        }

        state.inventory.last_click_ms = timestamp_ms;
        findings
    }
}
