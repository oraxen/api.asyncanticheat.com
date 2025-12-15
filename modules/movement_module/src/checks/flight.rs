//! Flight detection
//!
//! Detects:
//! - Y prediction violations (not following gravity)
//! - Sustained ascending (going up for too long)
//! - Hovering (staying at same Y level in air)

use crate::config::{FlightConfig, GRAVITY, DRAG};
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

pub struct FlightCheck {
    config: FlightConfig,
}

impl FlightCheck {
    pub fn new(config: FlightConfig) -> Self {
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

        // Only check on position packets
        let (y, on_ground) = match packet {
            ParsedPacket::Position(p) => (p.y, p.on_ground),
            ParsedPacket::PositionLook(p) => (p.y, p.on_ground),
            _ => return findings,
        };

        // Skip if recently teleported
        if state.movement.ticks_since_teleport < 5 {
            state.flight.last_y = y;
            return findings;
        }

        let y_delta = y - state.flight.last_y;
        
        // Track air ticks
        if on_ground {
            state.flight.air_ticks = 0;
            state.flight.ascend_ticks = 0;
            state.flight.hover_ticks = 0;
            state.flight.last_on_ground_y = y;
            state.flight.buffer_ypred.pass();
            state.flight.buffer_ascend.pass();
            state.flight.buffer_hover.pass();
        } else {
            state.flight.air_ticks += 1;

            // Only check after a few ticks in air
            if state.flight.air_ticks > 2 {
                // Check Y prediction
                findings.extend(self.check_y_prediction(state, y_delta, timestamp_ms));

                // Check sustained ascending
                findings.extend(self.check_sustained_ascend(state, y_delta, timestamp_ms));

                // Check hovering
                findings.extend(self.check_hover(state, y_delta, timestamp_ms));
            }

            // Calculate predicted next Y delta (apply gravity and drag)
            state.flight.predicted_y_delta = (state.flight.last_y_delta - GRAVITY) * DRAG;
        }

        state.flight.last_y = y;
        state.flight.last_y_delta = y_delta;

        findings
    }

    /// Check if Y movement matches gravity prediction
    fn check_y_prediction(
        &self,
        state: &mut PlayerState,
        y_delta: f64,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Only check after we have a prediction
        if state.flight.air_ticks < 3 {
            return findings;
        }

        let predicted = state.flight.predicted_y_delta;
        let deviation = (y_delta - predicted).abs();

        // Flying up when should be falling
        if y_delta > 0.0 && predicted < -0.1 && deviation > self.config.y_prediction_tolerance {
            let flagged = state.flight.buffer_ypred.fail_with(deviation);
            
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::FlightYPrediction,
                        deviation,
                        state.flight.buffer_ypred.vl(),
                        state.flight.buffer_ypred.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Y prediction violation: delta={:.4}, predicted={:.4}, dev={:.4}",
                        y_delta, predicted, deviation
                    ))
                    .with_evidence(serde_json::json!({
                        "y_delta": y_delta,
                        "predicted": predicted,
                        "deviation": deviation,
                        "air_ticks": state.flight.air_ticks
                    })),
                );
            }
        } else {
            state.flight.buffer_ypred.pass();
        }

        findings
    }

    /// Check for sustained upward movement
    fn check_sustained_ascend(
        &self,
        state: &mut PlayerState,
        y_delta: f64,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Track ascending ticks
        if y_delta > self.config.hover_threshold {
            state.flight.ascend_ticks += 1;
        } else {
            state.flight.ascend_ticks = 0;
        }

        // Check if ascending too long (normal jump peaks at ~8 ticks)
        if state.flight.ascend_ticks > self.config.max_ascend_ticks {
            let flagged = state.flight.buffer_ascend.fail();
            
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::FlightSustainedAscend,
                        state.flight.ascend_ticks as f64,
                        state.flight.buffer_ascend.vl(),
                        state.flight.buffer_ascend.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Sustained ascension: {} ticks (max: {})",
                        state.flight.ascend_ticks, self.config.max_ascend_ticks
                    ))
                    .with_evidence(serde_json::json!({
                        "ascend_ticks": state.flight.ascend_ticks,
                        "y_delta": y_delta,
                        "air_ticks": state.flight.air_ticks
                    })),
                );
            }
        }

        findings
    }

    /// Check for hovering (staying at same Y level)
    fn check_hover(
        &self,
        state: &mut PlayerState,
        y_delta: f64,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Track hover ticks (near-zero Y movement)
        if y_delta.abs() < self.config.hover_threshold && state.flight.air_ticks > 5 {
            state.flight.hover_ticks += 1;
        } else {
            state.flight.hover_ticks = 0;
        }

        // Check if hovering too long
        if state.flight.hover_ticks > self.config.max_hover_ticks {
            let flagged = state.flight.buffer_hover.fail();
            
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::FlightHover,
                        state.flight.hover_ticks as f64,
                        state.flight.buffer_hover.vl(),
                        state.flight.buffer_hover.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Hover detected: {} ticks (max: {})",
                        state.flight.hover_ticks, self.config.max_hover_ticks
                    ))
                    .with_evidence(serde_json::json!({
                        "hover_ticks": state.flight.hover_ticks,
                        "y_delta": y_delta,
                        "air_ticks": state.flight.air_ticks
                    })),
                );
            }
        }

        findings
    }
}
