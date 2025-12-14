//! Aimbot check (c7) - Aim modifications and combat rotation patterns
//!
//! Detects:
//! - Sensitivity mismatches / impossible mouse deltas
//! - Head snaps
//! - Suspicious pitch spread
//! - Zero-point / artificial rotation anchors

use crate::config::AimbotConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::{LookPacket, ParsedPacket, PositionLookPacket};
use crate::player_state::PlayerState;

/// Head snap threshold in degrees
const HEAD_SNAP_THRESHOLD: f32 = 30.0;
/// Minimum time between valid head snaps (ms)
const HEAD_SNAP_MIN_INTERVAL_MS: i64 = 50;
/// GCD (Greatest Common Divisor) for sensitivity detection
const SENSITIVITY_GCD_THRESHOLD: f64 = 0.01;
/// Maximum valid pitch
const MAX_PITCH: f32 = 90.0;
/// Minimum valid pitch
const MIN_PITCH: f32 = -90.0;

pub struct AimbotCheck {
    config: AimbotConfig,
}

impl AimbotCheck {
    pub fn new(config: AimbotConfig) -> Self {
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

        let (yaw, pitch) = match packet {
            ParsedPacket::Look(look) => (look.yaw, look.pitch),
            ParsedPacket::PositionLook(pos) => (pos.yaw, pos.pitch),
            _ => return Vec::new(),
        };

        // Calculate deltas
        let delta_yaw = self.normalize_angle(yaw - state.aimbot.last_yaw);
        let delta_pitch = pitch - state.aimbot.last_pitch;
        let time_delta = timestamp_ms - state.aimbot.last_rotation_ms;

        // Store samples
        if state.aimbot.last_rotation_ms > 0 {
            state.aimbot.yaw_deltas.push(delta_yaw.abs() as f64);
            state.aimbot.pitch_deltas.push(delta_pitch.abs() as f64);

            // Check head snap
            if self.config.check_head_snap {
                findings.extend(self.check_head_snap(state, delta_yaw, delta_pitch, time_delta, timestamp_ms));
            }

            // Check pitch spread
            if self.config.check_pitch_spread && state.aimbot.pitch_deltas.is_full() {
                findings.extend(self.check_pitch_spread(state, timestamp_ms));
            }

            // Check sensitivity
            if self.config.check_sensitivity && state.aimbot.yaw_deltas.is_full() {
                findings.extend(self.check_sensitivity(state, timestamp_ms));
            }

            // Check mouse delta
            if self.config.check_mouse_delta {
                findings.extend(self.check_mouse_delta(state, delta_yaw, delta_pitch, timestamp_ms));
            }

            // Check zero point - pass current yaw/pitch, not previous
            if self.config.check_zero_point {
                findings.extend(self.check_zero_point(state, yaw, pitch, delta_yaw, timestamp_ms));
            }
        }

        // Update state
        state.aimbot.last_yaw = yaw;
        state.aimbot.last_pitch = pitch;
        state.aimbot.last_rotation_ms = timestamp_ms;

        findings
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

    fn check_head_snap(
        &self,
        state: &mut PlayerState,
        delta_yaw: f32,
        delta_pitch: f32,
        time_delta: i64,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        let total_delta = (delta_yaw.powi(2) + delta_pitch.powi(2)).sqrt();

        // Check for sudden large rotation
        if total_delta > HEAD_SNAP_THRESHOLD && time_delta < HEAD_SNAP_MIN_INTERVAL_MS {
            state.aimbot.snap_count += 1;

            // Flag after multiple snaps
            if state.aimbot.snap_count >= 3 {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AacAimbotHeadSnap,
                        total_delta,
                        state.aimbot.snap_count as f32 / 10.0,
                        false,
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Head snap {:.1}° in {}ms (count: {})",
                        total_delta, time_delta, state.aimbot.snap_count
                    )),
                );
            }

            state.aimbot.last_snap_ms = timestamp_ms;
        } else if timestamp_ms - state.aimbot.last_snap_ms > 5000 {
            // Reset snap count after 5 seconds of no snaps
            state.aimbot.snap_count = 0;
        }

        findings
    }

    fn check_pitch_spread(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Calculate pitch delta variance
        let variance = state.aimbot.pitch_deltas.variance();
        let std_dev = variance.sqrt();

        // Very low variance in pitch deltas is suspicious (aimbot maintains constant pitch)
        if std_dev < 0.1 && state.aimbot.pitch_deltas.mean() > 0.5 {
            findings.push(
                Finding::new(
                    state.player_uuid,
                    FeatureId::AacAimbotPitchSpread,
                    std_dev as f32,
                    0.5,
                    false,
                    timestamp_ms,
                )
                .with_description(format!(
                    "Low pitch spread variance: {:.4} (mean delta: {:.2}°)",
                    std_dev,
                    state.aimbot.pitch_deltas.mean()
                )),
            );
        }

        findings
    }

    fn check_sensitivity(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        // GCD analysis for sensitivity detection
        let samples: Vec<f64> = state.aimbot.yaw_deltas.iter().copied().collect();
        if let Some(gcd) = self.calculate_gcd(&samples) {
            // Store for comparison
            state.aimbot.sensitivity_samples.push(gcd);

            if state.aimbot.sensitivity_samples.is_full() {
                let gcd_variance = state.aimbot.sensitivity_samples.variance();

                // Inconsistent GCD suggests external aim modification
                if gcd_variance > SENSITIVITY_GCD_THRESHOLD {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::AacAimbotSensMismatch,
                            gcd_variance as f32,
                            0.5,
                            false,
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "Sensitivity mismatch: GCD variance {:.6}",
                            gcd_variance
                        )),
                    );
                }
            }
        }

        findings
    }

    fn check_mouse_delta(
        &self,
        state: &mut PlayerState,
        delta_yaw: f32,
        delta_pitch: f32,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Check for impossible mouse deltas (non-integer pixel movements)
        // Mouse input should result in discrete angular changes based on sensitivity
        
        // Very small non-zero deltas are suspicious (below mouse precision)
        let min_delta = 0.01;
        if delta_yaw.abs() > 0.0 && delta_yaw.abs() < min_delta {
            findings.push(Finding::new(
                state.player_uuid,
                FeatureId::AacAimbotBadDeltaX,
                delta_yaw.abs(),
                0.3,
                false,
                timestamp_ms,
            ));
        }

        if delta_pitch.abs() > 0.0 && delta_pitch.abs() < min_delta {
            findings.push(Finding::new(
                state.player_uuid,
                FeatureId::AacAimbotBadDeltaY,
                delta_pitch.abs(),
                0.3,
                false,
                timestamp_ms,
            ));
        }

        findings
    }

    fn check_zero_point(
        &self,
        state: &mut PlayerState,
        current_yaw: f32,
        current_pitch: f32,
        delta_yaw: f32,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Zero-point detection: aimbot may return to exact angles repeatedly
        // Check if the player arrives at suspiciously round angles (current packet)
        let yaw_fraction = current_yaw.abs() % 1.0;
        let _pitch_fraction = current_pitch.abs() % 1.0;

        // Exact integer angles are rare in normal play - check CURRENT angle, not previous
        if yaw_fraction < 0.001 && delta_yaw.abs() > 1.0 {
            findings.push(
                Finding::new(
                    state.player_uuid,
                    FeatureId::AacAimbotZeroPoint,
                    current_yaw,
                    0.3,
                    false,
                    timestamp_ms,
                )
                .with_description(format!(
                    "Zero-point anchor at yaw {:.1}°",
                    current_yaw
                )),
            );
        }

        findings
    }

    fn calculate_gcd(&self, samples: &[f64]) -> Option<f64> {
        if samples.is_empty() {
            return None;
        }

        // Filter out zero values
        let non_zero: Vec<f64> = samples.iter().filter(|&&x| x > 0.001).copied().collect();
        if non_zero.is_empty() {
            return None;
        }

        // Simple GCD approximation using the smallest non-zero value
        let min = non_zero.iter().cloned().fold(f64::INFINITY, f64::min);
        Some(min)
    }
}

