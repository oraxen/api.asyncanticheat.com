//! Aim detection
//!
//! Detects:
//! - Head snaps (sudden large rotation changes)
//! - Pitch spread anomalies (aimbot maintains constant pitch)
//! - Sensitivity/GCD mismatches
//! - Modulo patterns (rotation snapping to divisions)
//! - Direction switching (instant reversal with large deltas)
//! - Repeated yaw values

use crate::config::AimConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

/// Modulo divisors for pattern detection
const MODULO_DIVISORS: [f64; 2] = [0.25, 0.1];

pub struct AimCheck {
    config: AimConfig,
}

impl AimCheck {
    pub fn new(config: AimConfig) -> Self {
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

        let (yaw, pitch) = match packet {
            ParsedPacket::Look(look) => (look.yaw, look.pitch),
            ParsedPacket::PositionLook(pos) => (pos.yaw, pos.pitch),
            _ => return Vec::new(),
        };

        let mut findings = Vec::new();

        if state.aim.last_rotation_ms > 0 {
            let time_delta = timestamp_ms - state.aim.last_rotation_ms;
            if time_delta <= 0 {
                return Vec::new(); // Out-of-order packet
            }

            let delta_yaw = self.normalize_angle(yaw - state.aim.last_yaw);
            let delta_pitch = pitch - state.aim.last_pitch;

            // Store samples
            state.aim.yaw_deltas.push(delta_yaw.abs() as f64);
            state.aim.pitch_deltas.push(delta_pitch.abs() as f64);
            state.aim.yaw_history.push_back(yaw);
            state.aim.pitch_history.push_back(pitch);
            if state.aim.yaw_history.len() > 20 {
                state.aim.yaw_history.pop_front();
                state.aim.pitch_history.pop_front();
            }

            // Only check during combat
            if state.combat.combat_ticks >= self.config.min_combat_ticks {
                // Head snap check
                findings.extend(self.check_head_snap(state, delta_yaw, delta_pitch, time_delta, timestamp_ms));

                // Pitch spread check
                if self.config.check_pitch_spread && state.aim.pitch_deltas.is_full() {
                    findings.extend(self.check_pitch_spread(state, timestamp_ms));
                }

                // Sensitivity/GCD check
                if self.config.check_sensitivity && state.aim.yaw_deltas.is_full() {
                    findings.extend(self.check_sensitivity(state, timestamp_ms));
                }

                // Modulo check
                if self.config.check_modulo {
                    findings.extend(self.check_modulo(state, delta_yaw, timestamp_ms));
                }

                // Direction switch check
                if self.config.check_direction_switch && state.aim.yaw_deltas.len() >= 2 {
                    findings.extend(self.check_direction_switch(state, timestamp_ms));
                }

                // Repeated yaw check
                if state.aim.yaw_history.len() >= 3 {
                    findings.extend(self.check_repeated_yaw(state, delta_yaw, timestamp_ms));
                }
            }
        }

        // Update state
        state.aim.last_yaw = yaw;
        state.aim.last_pitch = pitch;
        state.aim.last_rotation_ms = timestamp_ms;

        findings
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

        if total_delta > self.config.head_snap_threshold && time_delta < self.config.head_snap_min_interval_ms {
            state.aim.snap_count += 1;
            let flagged = state.aim.buffer_headsnap.fail();

            if flagged && state.aim.snap_count >= 3 {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AimHeadSnap,
                        total_delta as f64,
                        state.aim.buffer_headsnap.vl(),
                        state.aim.buffer_headsnap.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Head snap: {:.1}° in {}ms (count: {})",
                        total_delta, time_delta, state.aim.snap_count
                    )),
                );
            }
            state.aim.last_snap_ms = timestamp_ms;
        } else {
            state.aim.buffer_headsnap.pass();
            if timestamp_ms - state.aim.last_snap_ms > 5000 {
                state.aim.snap_count = 0;
            }
        }

        findings
    }

    fn check_pitch_spread(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        let std_dev = state.aim.pitch_deltas.std_dev();
        let mean = state.aim.pitch_deltas.mean();

        // Very low variance with significant movement is suspicious
        if std_dev < 0.1 && mean > 0.5 {
            let flagged = state.aim.buffer_pitch.fail();
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AimPitchSpread,
                        std_dev,
                        state.aim.buffer_pitch.vl(),
                        state.aim.buffer_pitch.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Low pitch spread: std_dev={:.4} mean={:.2}°",
                        std_dev, mean
                    )),
                );
            }
        } else {
            state.aim.buffer_pitch.pass();
        }

        findings
    }

    fn check_sensitivity(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        // GCD analysis for sensitivity detection
        let samples: Vec<f64> = state.aim.yaw_deltas.iter().copied().collect();
        if let Some(gcd) = self.calculate_gcd(&samples) {
            state.aim.sensitivity_samples.push(gcd);

            if state.aim.sensitivity_samples.is_full() {
                let gcd_variance = state.aim.sensitivity_samples.variance();

                // Inconsistent GCD suggests external aim modification
                if gcd_variance > 0.01 {
                    let flagged = state.aim.buffer_sens.fail();
                    if flagged {
                        findings.push(
                            Finding::new(
                                state.player_uuid,
                                FeatureId::AimSensitivity,
                                gcd_variance,
                                state.aim.buffer_sens.vl(),
                                state.aim.buffer_sens.max_vl(),
                                timestamp_ms,
                            )
                            .with_description(format!("Sensitivity mismatch: GCD variance {:.6}", gcd_variance)),
                        );
                    }
                } else {
                    state.aim.buffer_sens.pass();
                }
            }
        }

        findings
    }

    fn check_modulo(&self, state: &mut PlayerState, delta_yaw: f32, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        let delta_yaw_abs = delta_yaw.abs() as f64;
        let modulo_suspicious = MODULO_DIVISORS
            .iter()
            .any(|&d| delta_yaw_abs > 0.0 && (delta_yaw_abs % d).abs() < 0.0001);

        if modulo_suspicious {
            let flagged = state.aim.buffer_modulo.fail();
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AimModulo,
                        delta_yaw_abs,
                        state.aim.buffer_modulo.vl(),
                        state.aim.buffer_modulo.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!("Invalid yaw modulo: {:.4}°", delta_yaw_abs)),
                );
            }
        } else {
            state.aim.buffer_modulo.pass();
        }

        findings
    }

    fn check_direction_switch(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        let deltas = state.aim.yaw_deltas.last_n(2);
        if deltas.len() < 2 {
            return findings;
        }

        let current = deltas[0];
        let previous = deltas[1];
        let current_dir = current.signum();
        let prev_dir = previous.signum();

        // Direction switched with large deltas
        if current_dir != prev_dir && current > 30.0 && previous > 30.0 {
            let flagged = state.aim.buffer_dirswitch.fail();
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AimDirectionSwitch,
                        current,
                        state.aim.buffer_dirswitch.vl(),
                        state.aim.buffer_dirswitch.max_vl(),
                        timestamp_ms,
                    )
                    .with_description("Switching directions too quickly".to_string()),
                );
            }
        } else {
            state.aim.buffer_dirswitch.pass();
        }

        findings
    }

    fn check_repeated_yaw(&self, state: &mut PlayerState, delta_yaw: f32, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        let last_three: Vec<f32> = state.aim.yaw_history.iter().rev().take(3).cloned().collect();
        if last_three.len() < 3 {
            return findings;
        }

        // Three identical yaws with movement is suspicious
        if last_three[0] == last_three[1] && last_three[1] == last_three[2] && delta_yaw.abs() > 0.1 {
            let flagged = state.aim.buffer_repeated.fail();
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AimRepeatedYaw,
                        last_three[0] as f64,
                        state.aim.buffer_repeated.vl(),
                        state.aim.buffer_repeated.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!("Repeated yaw: {:.1}° three times", last_three[0])),
                );
            }
        } else {
            state.aim.buffer_repeated.pass();
        }

        findings
    }

    fn normalize_angle(&self, angle: f32) -> f32 {
        if !angle.is_finite() {
            return 0.0;
        }
        let mut a = angle % 360.0;
        if a > 180.0 {
            a -= 360.0;
        } else if a < -180.0 {
            a += 360.0;
        }
        a
    }

    fn calculate_gcd(&self, samples: &[f64]) -> Option<f64> {
        let non_zero: Vec<f64> = samples.iter().filter(|&&x| x > 0.001).copied().collect();
        if non_zero.is_empty() {
            return None;
        }
        Some(non_zero.iter().cloned().fold(f64::INFINITY, f64::min))
    }
}
