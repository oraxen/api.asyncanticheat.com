//! Movement checks: Flight, Speed, No Slow, Jump, Timer, Ground Spoof

use crate::config::VulcanConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::{Location, ParsedPacket};
use crate::player_state::PlayerState;

/// Movement thresholds
const GRAVITY: f64 = 0.08;
const DRAG: f64 = 0.98;
const GROUND_SPOOF_MOTION_THRESHOLD: f64 = 0.3116;
const MAX_WALK_SPEED: f64 = 0.2873;
const MAX_SPRINT_SPEED: f64 = 0.3675;
const MAX_SNEAK_SPEED: f64 = 0.0663;
const TIMER_EXPECTED_MS: f64 = 50.0; // 20 TPS
const TIMER_THRESHOLD: f64 = 1.01; // 1% faster
const FLYING_PACKET_LIMIT: u32 = 20;
const STEP_HEIGHT_LIMIT: f64 = 0.6;

pub struct MovementChecks {
    config: VulcanConfig,
}

impl MovementChecks {
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
            ParsedPacket::Position(pos) => {
                let loc = Location {
                    x: pos.x,
                    y: pos.y,
                    z: pos.z,
                    yaw: state.movement.last_location.map(|l| l.yaw).unwrap_or(0.0),
                    pitch: state.movement.last_location.map(|l| l.pitch).unwrap_or(0.0),
                    on_ground: pos.on_ground,
                };
                findings.extend(self.check_movement(state, loc, timestamp_ms));
                state.movement.flying_packet_count = 0;
            }
            ParsedPacket::PositionLook(pos) => {
                let loc = Location {
                    x: pos.x,
                    y: pos.y,
                    z: pos.z,
                    yaw: pos.yaw,
                    pitch: pos.pitch,
                    on_ground: pos.on_ground,
                };
                findings.extend(self.check_movement(state, loc, timestamp_ms));
                state.movement.flying_packet_count = 0;
            }
            ParsedPacket::Flying(flying) => {
                state.movement.flying_packet_count += 1;
                
                // Bad Packets B - Flying packet flood
                if state.movement.flying_packet_count > FLYING_PACKET_LIMIT {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::BadPacketsB,
                            state.movement.flying_packet_count as f64,
                            1,
                            1,
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "{} flying packets in a row (limit: {})",
                            state.movement.flying_packet_count, FLYING_PACKET_LIMIT
                        )),
                    );
                    state.player_violations += 1;
                }
            }
            ParsedPacket::EntityAction(action) => {
                match action.action.as_str() {
                    "START_SPRINTING" => state.movement.is_sprinting = true,
                    "STOP_SPRINTING" => state.movement.is_sprinting = false,
                    "START_SNEAKING" => state.movement.is_sneaking = true,
                    "STOP_SNEAKING" => state.movement.is_sneaking = false,
                    _ => {}
                }
            }
            _ => {}
        }

        findings
    }

    fn check_movement(
        &self,
        state: &mut PlayerState,
        new_loc: Location,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        state.ticks_existed += 1;
        state.movement.tick_count += 1;

        // Timer check
        state.movement.packet_timestamps.push(timestamp_ms as f64);
        findings.extend(self.check_timer(state, timestamp_ms));

        if let Some(last_loc) = state.movement.last_location {
            let motion_y = new_loc.y - last_loc.y;
            let h_dist = new_loc.horizontal_distance(&last_loc);

            state.movement.last_motion_y = state.movement.motion_y;
            state.movement.motion_y = motion_y;

            // Flight checks
            if self.config.flight.enabled {
                findings.extend(self.check_flight(state, &new_loc, motion_y, timestamp_ms));
            }

            // Speed checks
            if self.config.speed.enabled {
                findings.extend(self.check_speed(state, h_dist, timestamp_ms));
            }

            // Ground Spoof checks
            if self.config.groundspoof.enabled && state.ticks_existed >= 20 {
                findings.extend(self.check_ground_spoof(state, &new_loc, motion_y, timestamp_ms));
            }

            // Step check
            if self.config.step.enabled {
                findings.extend(self.check_step(state, &last_loc, &new_loc, motion_y, timestamp_ms));
            }

            // Update prediction for next tick
            self.update_prediction(state, motion_y);
        }

        // Update state
        state.movement.last_location = Some(new_loc);
        state.movement.last_on_ground = new_loc.on_ground;
        state.movement.last_move_ms = timestamp_ms;

        findings
    }

    /// Flight checks (A, B, C, D, E)
    fn check_flight(
        &self,
        state: &mut PlayerState,
        new_loc: &Location,
        motion_y: f64,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Skip if in special states
        if state.movement.in_liquid || state.movement.on_ladder || 
           state.movement.using_elytra || state.movement.in_vehicle {
            return findings;
        }

        // Flight A - Server-side Y prediction
        if self.config.flight.a.enabled && state.movement.last_location.is_some() {
            let predicted = state.movement.predicted_y;
            let actual = motion_y;
            let diff = (actual - predicted).abs();

            // Allow some tolerance
            if diff > 0.1 && !new_loc.on_ground && !state.movement.last_on_ground {
                let flagged = state.movement.flight_buffer.fail_with(diff);
                if flagged {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::FlightA,
                            state.movement.flight_buffer.get(),
                            state.movement.flight_buffer.vl(),
                            state.movement.flight_buffer.max_vl(),
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "Invalid Y prediction: expected {:.4}, got {:.4} (diff: {:.4})",
                            predicted, actual, diff
                        )),
                    );
                    state.movement_violations += 1;
                }
            } else {
                state.movement.flight_buffer.pass();
            }
        }

        // Flight C - Sustained ascension
        if self.config.flight.c.enabled && motion_y > 0.0 && state.movement.last_motion_y > 0.0 {
            // Ascending for multiple ticks without jump is suspicious
            if motion_y > 0.1 && !new_loc.on_ground {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::FlightC,
                        motion_y,
                        1,
                        5,
                        timestamp_ms,
                    )
                    .with_description(format!("Sustained ascension: +{:.4} blocks/tick", motion_y)),
                );
                state.movement_violations += 1;
            }
        }

        // Flight E - Hover detection
        if self.config.flight.e.enabled && !new_loc.on_ground {
            if motion_y.abs() < 0.005 && state.movement.motion_y.abs() < 0.005 {
                // Near-zero vertical movement while in air
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::FlightE,
                        motion_y.abs(),
                        1,
                        5,
                        timestamp_ms,
                    )
                    .with_description("Hovering in air".to_string()),
                );
                state.movement_violations += 1;
            }
        }

        findings
    }

    /// Speed checks (A, B, C, D)
    fn check_speed(
        &self,
        state: &mut PlayerState,
        h_dist: f64,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Determine max allowed speed based on state
        let base_max = if state.movement.is_sneaking {
            MAX_SNEAK_SPEED
        } else if state.movement.is_sprinting {
            MAX_SPRINT_SPEED
        } else {
            MAX_WALK_SPEED
        };

        // Add tolerance for potion effects, velocity, etc.
        let max_speed = base_max * 1.5; // 50% tolerance

        if h_dist > max_speed {
            let ratio = h_dist / base_max;
            let flagged = state.movement.speed_buffer.fail_with(ratio - 1.0);
            
            if flagged {
                let feature = if state.movement.last_on_ground {
                    FeatureId::SpeedB
                } else {
                    FeatureId::SpeedC
                };

                findings.push(
                    Finding::new(
                        state.player_uuid,
                        feature,
                        state.movement.speed_buffer.get(),
                        state.movement.speed_buffer.vl(),
                        state.movement.speed_buffer.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Speed: {:.4} b/t (max: {:.4}, {:.1}x)",
                        h_dist, base_max, ratio
                    )),
                );
                state.movement_violations += 1;
            }
        } else {
            state.movement.speed_buffer.pass();
        }

        findings
    }

    /// Ground Spoof checks (A, B, C)
    fn check_ground_spoof(
        &self,
        state: &mut PlayerState,
        new_loc: &Location,
        motion_y: f64,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Ground Spoof A - Claiming ground while falling fast
        if new_loc.on_ground && motion_y.abs() > GROUND_SPOOF_MOTION_THRESHOLD {
            let flagged = state.movement.groundspoof_buffer.fail();
            if flagged {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::GroundSpoofA,
                        state.movement.groundspoof_buffer.get(),
                        state.movement.groundspoof_buffer.vl(),
                        state.movement.groundspoof_buffer.max_vl(),
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Spoofed ground: motionY={:.4} (threshold: {:.4})",
                        motion_y, GROUND_SPOOF_MOTION_THRESHOLD
                    )),
                );
                state.movement_violations += 1;
            }
        } else {
            state.movement.groundspoof_buffer.pass();
        }

        // Ground Spoof B - Falling while claiming ground
        if new_loc.on_ground && motion_y < -0.2 {
            findings.push(
                Finding::new(
                    state.player_uuid,
                    FeatureId::GroundSpoofB,
                    motion_y.abs(),
                    1,
                    5,
                    timestamp_ms,
                )
                .with_description(format!("Falling while on ground: {:.4}", motion_y)),
            );
            state.movement_violations += 1;
        }

        findings
    }

    /// Step check (vanilla max 0.5 blocks)
    fn check_step(
        &self,
        state: &mut PlayerState,
        last_loc: &Location,
        new_loc: &Location,
        motion_y: f64,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Step A - Invalid step height
        if last_loc.on_ground && new_loc.on_ground && motion_y > STEP_HEIGHT_LIMIT {
            findings.push(
                Finding::new(
                    state.player_uuid,
                    FeatureId::StepA,
                    motion_y,
                    1,
                    5,
                    timestamp_ms,
                )
                .with_description(format!(
                    "Invalid step: {:.4} blocks (max: {:.1})",
                    motion_y, STEP_HEIGHT_LIMIT
                )),
            );
            state.movement_violations += 1;
        }

        findings
    }

    /// Timer check (game speed manipulation)
    fn check_timer(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        if !self.config.timer.enabled {
            return findings;
        }

        if state.movement.packet_timestamps.is_full() {
            // Calculate average delay between packets
            let timestamps: Vec<f64> = state.movement.packet_timestamps.iter().cloned().collect();
            let mut delays: Vec<f64> = Vec::new();
            
            for i in 1..timestamps.len() {
                delays.push(timestamps[i] - timestamps[i - 1]);
            }

            if !delays.is_empty() {
                let avg_delay: f64 = delays.iter().sum::<f64>() / delays.len() as f64;
                let speed = TIMER_EXPECTED_MS / avg_delay;

                if speed > TIMER_THRESHOLD {
                    let flagged = state.movement.timer_buffer.fail_with(speed - 1.0);
                    if flagged {
                        findings.push(
                            Finding::new(
                                state.player_uuid,
                                FeatureId::TimerA,
                                state.movement.timer_buffer.get(),
                                state.movement.timer_buffer.vl(),
                                state.movement.timer_buffer.max_vl(),
                                timestamp_ms,
                            )
                            .with_description(format!(
                                "Timer: {:.1}% faster (avg delay: {:.1}ms)",
                                (speed - 1.0) * 100.0,
                                avg_delay
                            )),
                        );
                        state.movement_violations += 1;
                    }
                } else {
                    state.movement.timer_buffer.pass();
                }
            }
        }

        findings
    }

    /// Update Y prediction for next tick
    fn update_prediction(&self, state: &mut PlayerState, motion_y: f64) {
        // Simple gravity prediction: nextMotion = (motion - gravity) * drag
        let next_motion = (motion_y - GRAVITY) * DRAG;
        state.movement.predicted_y = next_motion;
    }
}

