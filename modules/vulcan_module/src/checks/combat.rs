//! Combat checks: Aim, Auto Clicker, Velocity, Reach, Hitbox, Kill Aura, Criticals

use crate::config::VulcanConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::{ParsedPacket, UseEntityPacket};
use crate::player_state::PlayerState;

/// Combat check thresholds
const AIM_SLOPE_THRESHOLD: f32 = 0.5;
const AIM_MODULO_DIVISORS: [f64; 2] = [0.25, 0.1];
const CPS_WINDOW_MS: i64 = 1000;
const CLICK_SAMPLE_SIZE: usize = 20;
const STD_DEV_THRESHOLD: f64 = 167.0;
const VELOCITY_RATIO_THRESHOLD: f64 = 0.999;
const VELOCITY_JUMP_THRESHOLD: f64 = 0.419999;
const REACH_BASE: f64 = 3.0;
const KILLAURA_POST_THRESHOLD_MS: i64 = 5;

pub struct CombatChecks {
    config: VulcanConfig,
}

impl CombatChecks {
    pub fn new(config: VulcanConfig) -> Self {
        Self { config }
    }

    pub fn process(
        &self,
        state: &mut PlayerState,
        packet: &ParsedPacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        match packet {
            ParsedPacket::Look(look) => {
                findings.extend(self.check_aim(state, look.yaw, look.pitch, timestamp_ms));
            }
            ParsedPacket::PositionLook(pos) => {
                findings.extend(self.check_aim(state, pos.yaw, pos.pitch, timestamp_ms));
            }
            ParsedPacket::UseEntity(use_entity) => {
                if use_entity.action == "ATTACK" {
                    findings.extend(self.check_attack(state, use_entity, timestamp_ms));
                }
            }
            ParsedPacket::ArmAnimation(_) => {
                self.handle_swing(state, timestamp_ms);
            }
            ParsedPacket::EntityVelocity(vel) => {
                self.handle_velocity(state, vel.velocity_x, vel.velocity_y, vel.velocity_z, timestamp_ms);
            }
            _ => {}
        }

        findings
    }

    /// Aim checks (A, B, C, L, etc.)
    fn check_aim(
        &self,
        state: &mut PlayerState,
        yaw: f32,
        pitch: f32,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        if !self.config.aim.a.enabled {
            return findings;
        }

        // Update combat state
        if state.combat.in_combat {
            state.combat.combat_ticks += 1;
        }

        if state.aim.last_rotation_ms > 0 {
            let delta_yaw = self.normalize_angle(yaw - state.aim.last_yaw);
            let delta_pitch = pitch - state.aim.last_pitch;
            let time_delta = timestamp_ms - state.aim.last_rotation_ms;

            // Store deltas
            state.aim.yaw_deltas.push(delta_yaw.abs() as f64);
            state.aim.pitch_deltas.push(delta_pitch.abs() as f64);
            state.aim.yaw_history.push_back(yaw);
            state.aim.pitch_history.push_back(pitch);
            if state.aim.yaw_history.len() > 20 {
                state.aim.yaw_history.pop_front();
                state.aim.pitch_history.pop_front();
            }

            // Only check during combat (>= 3 ticks as per Vulcan)
            if state.combat.combat_ticks >= 3 {
                // Aim A - Slope check
                if self.config.aim.a.enabled && delta_pitch.abs() > AIM_SLOPE_THRESHOLD {
                    let flagged = state.aim.buffer_a.fail();
                    if flagged {
                        findings.push(
                            Finding::new(
                                state.player_uuid,
                                FeatureId::AimA,
                                state.aim.buffer_a.get(),
                                state.aim.buffer_a.vl(),
                                state.aim.buffer_a.max_vl(),
                                timestamp_ms,
                            )
                            .with_description(format!(
                                "Invalid pitch change: {:.2}° (threshold: {:.1}°)",
                                delta_pitch.abs(),
                                AIM_SLOPE_THRESHOLD
                            )),
                        );
                        state.combat_violations += 1;
                    }
                } else {
                    state.aim.buffer_a.pass();
                }

                // Aim B - Modulo check
                if self.config.aim.b.enabled {
                    let delta_yaw_abs = delta_yaw.abs() as f64;
                    let modulo_suspicious = AIM_MODULO_DIVISORS
                        .iter()
                        .any(|&d| delta_yaw_abs > 0.0 && (delta_yaw_abs % d).abs() < 0.0001);

                    if modulo_suspicious {
                        let flagged = state.aim.buffer_b.fail();
                        if flagged {
                            findings.push(
                                Finding::new(
                                    state.player_uuid,
                                    FeatureId::AimB,
                                    state.aim.buffer_b.get(),
                                    state.aim.buffer_b.vl(),
                                    state.aim.buffer_b.max_vl(),
                                    timestamp_ms,
                                )
                                .with_description(format!(
                                    "Invalid yaw modulo: {:.4}°",
                                    delta_yaw_abs
                                )),
                            );
                            state.combat_violations += 1;
                        }
                    } else {
                        state.aim.buffer_b.pass();
                    }
                }

                // Aim C - Repeated yaw
                if self.config.aim.c.enabled && state.aim.yaw_history.len() >= 3 {
                    let last_three: Vec<f32> = state.aim.yaw_history.iter().rev().take(3).cloned().collect();
                    if last_three[0] == last_three[1] && last_three[1] == last_three[2] && delta_yaw.abs() > 0.1 {
                        let flagged = state.aim.buffer_c.fail();
                        if flagged {
                            findings.push(Finding::new(
                                state.player_uuid,
                                FeatureId::AimC,
                                state.aim.buffer_c.get(),
                                state.aim.buffer_c.vl(),
                                state.aim.buffer_c.max_vl(),
                                timestamp_ms,
                            ));
                            state.combat_violations += 1;
                        }
                    } else {
                        state.aim.buffer_c.pass();
                    }
                }

                // Aim L - Direction switching too quickly
                if self.config.aim.l.enabled && state.aim.yaw_deltas.len() >= 2 {
                    let deltas = state.aim.yaw_deltas.last_n(2);
                    let current_dir = deltas.last().map(|d| d.signum()).unwrap_or(0.0);
                    let prev_dir = deltas.first().map(|d| d.signum()).unwrap_or(0.0);
                    
                    let d0 = deltas.last().copied().unwrap_or(0.0);
                    let d1 = deltas.first().copied().unwrap_or(0.0);
                    if current_dir != prev_dir && d0.abs() > 30.0 && d1.abs() > 30.0 {
                        let flagged = state.aim.buffer_l.fail();
                        if flagged {
                            findings.push(
                                Finding::new(
                                    state.player_uuid,
                                    FeatureId::AimL,
                                    state.aim.buffer_l.get(),
                                    state.aim.buffer_l.vl(),
                                    state.aim.buffer_l.max_vl(),
                                    timestamp_ms,
                                )
                                .with_description("Switching directions too quickly".to_string()),
                            );
                            state.combat_violations += 1;
                        }
                    } else {
                        state.aim.buffer_l.pass();
                    }
                }
            }
        }

        state.aim.last_yaw = yaw;
        state.aim.last_pitch = pitch;
        state.aim.last_rotation_ms = timestamp_ms;

        findings
    }

    /// Attack checks (Auto Clicker, Kill Aura, Reach, Hitbox, Criticals)
    fn check_attack(
        &self,
        state: &mut PlayerState,
        use_entity: &UseEntityPacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Enter combat
        state.combat.in_combat = true;

        // Kill Aura A - Post check
        if self.config.killaura.a.enabled && state.combat.last_attack_ms > 0 {
            let attack_delay = timestamp_ms - state.combat.last_attack_ms;
            if attack_delay < KILLAURA_POST_THRESHOLD_MS {
                state.combat.rapid_attack = true;
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::KillAuraA,
                        1.0,
                        1,
                        5,
                        timestamp_ms,
                    )
                    .with_description(format!("Post attack: {}ms delay", attack_delay)),
                );
                state.combat_violations += 1;
            }
        }

        // Kill Aura D - Multi Aura
        if self.config.killaura.d.enabled {
            if let Some(last_target) = state.combat.last_attack_target {
                if last_target != use_entity.entity_id {
                    let target_switch_time = timestamp_ms - state.combat.last_attack_ms;
                    if target_switch_time < 50 {
                        findings.push(
                            Finding::new(
                                state.player_uuid,
                                FeatureId::KillAuraD,
                                1.0,
                                1,
                                5,
                                timestamp_ms,
                            )
                            .with_description(format!(
                                "Multi-aura: switched targets in {}ms",
                                target_switch_time
                            )),
                        );
                        state.combat_violations += 1;
                    }
                }
            }
        }

        // Auto Clicker checks
        findings.extend(self.check_autoclicker(state, timestamp_ms));

        // Bad Packets 9 - No Swing
        if self.config.badpackets.enabled {
            let swing_age = timestamp_ms - state.combat.last_swing_ms;
            if swing_age > 500 || state.combat.last_swing_ms == 0 {
                state.combat.attacks_since_swing += 1;
                if state.combat.attacks_since_swing >= 3 {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::BadPackets9,
                            state.combat.attacks_since_swing as f64,
                            1,
                            1,
                            timestamp_ms,
                        )
                        .with_description("Attacking without swinging arm".to_string()),
                    );
                    state.player_violations += 1;
                }
            }
        }

        state.combat.last_attack_ms = timestamp_ms;
        state.combat.last_attack_target = Some(use_entity.entity_id);

        findings
    }

    /// Auto Clicker statistical analysis
    fn check_autoclicker(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        if !self.config.autoclicker.enabled {
            return findings;
        }

        // Record click interval
        if state.autoclicker.last_click_ms > 0 {
            let interval = (timestamp_ms - state.autoclicker.last_click_ms) as f64;
            state.autoclicker.click_intervals.push(interval);
        }

        // Update CPS tracking
        if state.autoclicker.window_start_ms == 0 {
            state.autoclicker.window_start_ms = timestamp_ms;
        }
        state.autoclicker.clicks_in_window += 1;

        let window_elapsed = timestamp_ms - state.autoclicker.window_start_ms;
        if window_elapsed >= CPS_WINDOW_MS {
            let cps = (state.autoclicker.clicks_in_window as f64 * 1000.0) / window_elapsed as f64;
            state.autoclicker.last_stats.cps = cps;

            // Auto Clicker A - CPS limit
            if cps > self.config.autoclicker.cps_limit {
                let flagged = state.autoclicker.buffer_a.fail();
                if flagged {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::AutoClickerA,
                            state.autoclicker.buffer_a.get(),
                            state.autoclicker.buffer_a.vl(),
                            state.autoclicker.buffer_a.max_vl(),
                            timestamp_ms,
                        )
                        .with_description(format!("{:.1} CPS (limit: {:.0})", cps, self.config.autoclicker.cps_limit)),
                    );
                    state.combat_violations += 1;
                }
            } else {
                state.autoclicker.buffer_a.pass();
            }

            state.autoclicker.clicks_in_window = 0;
            state.autoclicker.window_start_ms = timestamp_ms;
        }

        // Statistical analysis when buffer is full
        if state.autoclicker.click_intervals.is_full() {
            let std_dev = state.autoclicker.click_intervals.std_dev();
            let variance = state.autoclicker.click_intervals.variance();
            let kurtosis = state.autoclicker.click_intervals.kurtosis();
            let distinct = state.autoclicker.click_intervals.distinct_count();

            state.autoclicker.last_stats.std_dev = std_dev;
            state.autoclicker.last_stats.variance = variance;
            state.autoclicker.last_stats.kurtosis = kurtosis;
            state.autoclicker.last_stats.distinct = distinct;

            // Auto Clicker B - Low deviation
            if std_dev < STD_DEV_THRESHOLD && state.autoclicker.last_stats.cps > 5.0 {
                let flagged = state.autoclicker.buffer_b.fail();
                if flagged {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::AutoClickerB,
                            state.autoclicker.buffer_b.get(),
                            state.autoclicker.buffer_b.vl(),
                            state.autoclicker.buffer_b.max_vl(),
                            timestamp_ms,
                        )
                        .with_description(format!("Low std dev: {:.1}ms (threshold: {:.0}ms)", std_dev, STD_DEV_THRESHOLD)),
                    );
                    state.combat_violations += 1;
                }
            } else {
                state.autoclicker.buffer_b.pass();
            }

            // Auto Clicker E - Low variance
            if variance < 2000.0 && state.autoclicker.last_stats.cps > 8.0 {
                let flagged = state.autoclicker.buffer_e.fail();
                if flagged {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::AutoClickerE,
                            state.autoclicker.buffer_e.get(),
                            state.autoclicker.buffer_e.vl(),
                            state.autoclicker.buffer_e.max_vl(),
                            timestamp_ms,
                        )
                        .with_description(format!("Low variance: {:.1}", variance)),
                    );
                    state.combat_violations += 1;
                }
            } else {
                state.autoclicker.buffer_e.pass();
            }

            // Auto Clicker I - Low kurtosis
            if kurtosis < -1.0 && state.autoclicker.last_stats.cps > 8.0 {
                let flagged = state.autoclicker.buffer_i.fail();
                if flagged {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::AutoClickerI,
                            state.autoclicker.buffer_i.get(),
                            state.autoclicker.buffer_i.vl(),
                            state.autoclicker.buffer_i.max_vl(),
                            timestamp_ms,
                        )
                        .with_description(format!("Low kurtosis: {:.2}", kurtosis)),
                    );
                    state.combat_violations += 1;
                }
            } else {
                state.autoclicker.buffer_i.pass();
            }
        }

        state.autoclicker.last_click_ms = timestamp_ms;
        findings
    }

    fn handle_swing(&self, state: &mut PlayerState, timestamp_ms: i64) {
        state.combat.last_swing_ms = timestamp_ms;
        state.combat.attacks_since_swing = 0;
    }

    fn handle_velocity(&self, state: &mut PlayerState, vx: f64, vy: f64, vz: f64, timestamp_ms: i64) {
        state.velocity.pending_velocity = Some((vx, vy, vz));
        state.velocity.velocity_received_ms = timestamp_ms;
        state.velocity.ticks_since_velocity = 0;
    }

    fn normalize_angle(&self, angle: f32) -> f32 {
        // Guard against non-finite values (NaN/±Inf) to avoid infinite loops.
        if !angle.is_finite() {
            return 0.0;
        }
        // Fast normalization into [-180, 180] without while-loops.
        let mut a = angle % 360.0;
        if a > 180.0 {
            a -= 360.0;
        } else if a < -180.0 {
            a += 360.0;
        }
        a
    }
}

