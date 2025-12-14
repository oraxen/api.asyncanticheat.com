//! Movement check (ca_0) - Timer, NoFall, Velocity, NoSlow, etc.
//!
//! A broad movement pipeline covering:
//! - Timer (client running faster)
//! - NoFall (invalid ground / fall damage avoidance)
//! - Velocity (ignoring knockback)
//! - NoSlow (movement not slowed while sneaking/using items)
//! - Generic movement anomalies

use crate::config::MoveConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::{EntityVelocityPacket, Location, ParsedPacket, SneakPacket};
use crate::player_state::PlayerState;

/// Maximum horizontal speed per tick (blocks) - vanilla walking is ~0.1
const MAX_WALK_SPEED: f64 = 0.3;
/// Maximum sprint speed per tick
const MAX_SPRINT_SPEED: f64 = 0.4;
/// Maximum sneak speed per tick
const MAX_SNEAK_SPEED: f64 = 0.13;
/// Timer check window (ms)
const TIMER_WINDOW_MS: i64 = 1000;
/// Expected moves per second at 20 TPS
const EXPECTED_MOVES_PER_SECOND: f64 = 20.0;
/// Timer tolerance percentage
const TIMER_TOLERANCE: f64 = 1.1; // 10% tolerance

/// Basic survival-fly heuristic: seconds airborne above last ground level before we consider it suspicious.
const FLY_AIRTIME_MS: i64 = 6000;
/// Require being at least this many blocks above last ground Y.
const FLY_MIN_ABOVE_GROUND: f64 = 3.0;

pub struct MovementCheck {
    config: MoveConfig,
}

impl MovementCheck {
    pub fn new(config: MoveConfig) -> Self {
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
            }
            ParsedPacket::Look(look) => {
                // Update yaw/pitch from Look packets to keep state.movement.last_location accurate
                // This is important for InteractCheck and other checks that rely on look direction
                if let Some(ref mut loc) = state.movement.last_location {
                    loc.yaw = look.yaw;
                    loc.pitch = look.pitch;
                }
            }
            ParsedPacket::EntityVelocity(vel) => {
                findings.extend(self.handle_velocity(state, vel, timestamp_ms));
            }
            ParsedPacket::Sneak(sneak) => {
                self.handle_sneak(state, sneak);
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

        // Update move count for timer check
        state.movement.move_count += 1;

        // Timer check
        if state.movement.timer_start_ms.is_none() {
            state.movement.timer_start_ms = Some(timestamp_ms);
        } else {
            findings.extend(self.check_timer(state, timestamp_ms));
        }

        // Track air window early so movement-only heuristics can use it.
        if new_loc.on_ground {
            state.movement.air_start_ms = None;
            state.movement.max_air_y = new_loc.y;
        } else {
            if state.movement.air_start_ms.is_none() {
                state.movement.air_start_ms = Some(timestamp_ms);
                state.movement.max_air_y = new_loc.y;
            } else if new_loc.y > state.movement.max_air_y {
                state.movement.max_air_y = new_loc.y;
            }
        }

        if let Some(last_loc) = state.movement.last_location {
            // Horizontal distance check
            let h_dist = new_loc.horizontal_distance(&last_loc);
            let v_dist = new_loc.y - last_loc.y;

            // Check NoSlow
            if self.config.check_sneak && state.movement.is_sneaking {
                findings.extend(self.check_noslow_sneak(state, h_dist, timestamp_ms));
            }
            if self.config.check_item_use && state.movement.using_item {
                findings.extend(self.check_noslow_item(state, h_dist, timestamp_ms));
            }

            // Check NoFall
            if self.config.block_nofall {
                findings.extend(self.check_nofall(state, &new_loc, v_dist, timestamp_ms));
            }

            // Check velocity usage
            if state.movement.pending_velocity.is_some() {
                findings.extend(self.check_velocity(state, h_dist, v_dist, timestamp_ms));
            }

            // Generic movement check
            findings.extend(self.check_generic_movement(state, h_dist, timestamp_ms));

            // Basic survival fly / hover detection (movement-only).
            findings.extend(self.check_survival_fly(state, &new_loc, v_dist, timestamp_ms));
        }

        // Track fall distance BEFORE updating last_location
        // (otherwise we'd compare new_loc to itself)
        if new_loc.on_ground {
            // Player landed: reset fall distance regardless of magnitude to avoid small-fall accumulation.
                state.movement.fall_distance = 0.0;
            state.movement.last_ground_y = new_loc.y;
        } else if let Some(last) = state.movement.last_location {
            if new_loc.y < last.y {
                state.movement.fall_distance += last.y - new_loc.y;
            }
        }

        // Update state after fall distance tracking
        state.movement.last_location = Some(new_loc);
        state.movement.last_on_ground = new_loc.on_ground;
        state.movement.last_move_ms = timestamp_ms;

        findings
    }

    fn check_survival_fly(
        &self,
        state: &mut PlayerState,
        new_loc: &Location,
        v_dist: f64,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        if new_loc.on_ground {
            return findings;
        }

        let Some(start_ms) = state.movement.air_start_ms else {
            return findings;
        };

        let air_ms = timestamp_ms - start_ms;
        if air_ms <= 0 {
            return findings;
        }

        let above_ground = new_loc.y - state.movement.last_ground_y;
        if air_ms >= FLY_AIRTIME_MS && above_ground >= FLY_MIN_ABOVE_GROUND {
            // If we're still not falling (or descending extremely slowly), it's suspicious.
            // Keep conservative to avoid spam/false positives.
            if v_dist >= -0.25 {
                let mitigated = state.movement.distance_vl.update(1.0, timestamp_ms);
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AacMoveGeneric,
                        (air_ms as f32) / 1000.0,
                        state.movement.distance_vl.get(),
                        mitigated,
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Suspicious airtime: {:.1}s airborne at y={:.2} (+{:.1} above last ground y={:.2})",
                        air_ms as f64 / 1000.0,
                        new_loc.y,
                        above_ground,
                        state.movement.last_ground_y
                    )),
                );

                // Reset window so we don't emit every tick.
                state.movement.air_start_ms = Some(timestamp_ms);
                state.movement.max_air_y = new_loc.y;
            }
        }

        findings
    }

    fn check_timer(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        let Some(start_ms) = state.movement.timer_start_ms else {
            state.movement.timer_start_ms = Some(timestamp_ms);
            state.movement.move_count = 0;
            return findings;
        };

        let elapsed_ms = timestamp_ms - start_ms;
        // Out-of-order timestamps: reset the window instead of producing nonsense ratios.
        if elapsed_ms <= 0 {
            state.movement.timer_start_ms = Some(timestamp_ms);
            state.movement.move_count = 0;
            return findings;
        }

        if elapsed_ms >= TIMER_WINDOW_MS {
            let expected_moves = (elapsed_ms as f64 / 1000.0) * EXPECTED_MOVES_PER_SECOND;
            let actual_moves = state.movement.move_count as f64;
            let ratio = actual_moves / expected_moves;

            if ratio > TIMER_TOLERANCE {
                let advantage = ratio - 1.0;
                let mitigated = state.movement.timer_vl.update(advantage as f32, timestamp_ms);

                if ratio > 1.15 {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::AacMoveTimer,
                            advantage as f32,
                            state.movement.timer_vl.get(),
                            mitigated,
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "{:.0} moves in {:.1}s ({:.1}% faster)",
                            actual_moves,
                            elapsed_ms as f64 / 1000.0,
                            advantage * 100.0
                        )),
                    );
                }
            }

            // Reset timer window
            state.movement.move_count = 0;
            state.movement.timer_start_ms = Some(timestamp_ms);
        }

        findings
    }

    fn check_nofall(
        &self,
        state: &mut PlayerState,
        new_loc: &Location,
        v_dist: f64,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Check for invalid ground claim after falling
        if new_loc.on_ground && !state.movement.last_on_ground {
            // Player claims to have landed
            if state.movement.fall_distance > 3.0 {
                // Should take fall damage
                // Check if the ground claim is valid (Y position should be at block boundary)
                let y_frac = new_loc.y - new_loc.y.floor();
                
                // Valid landing should be near a block boundary
                if y_frac > 0.01 && y_frac < 0.99 {
                    let mitigated = state.movement.distance_vl.update(1.0, timestamp_ms);
                    
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::AacMoveNofall,
                            state.movement.fall_distance as f32,
                            state.movement.distance_vl.get(),
                            mitigated,
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "Invalid ground at y={:.2} after falling {:.1} blocks",
                            new_loc.y, state.movement.fall_distance
                        )),
                    );
                }
            }
        }

        // Check for falling while claiming ground
        if new_loc.on_ground && v_dist < -0.5 {
            let mitigated = state.movement.distance_vl.update(0.5, timestamp_ms);
            
            findings.push(Finding::new(
                state.player_uuid,
                FeatureId::AacMoveNofall,
                v_dist.abs() as f32,
                state.movement.distance_vl.get(),
                mitigated,
                timestamp_ms,
            ));
        }

        findings
    }

    fn check_velocity(
        &self,
        state: &mut PlayerState,
        h_dist: f64,
        v_dist: f64,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        if let Some((vx, vy, vz)) = state.movement.pending_velocity {
            let vel_age_ms = timestamp_ms - state.movement.velocity_received_ms;
            let max_vel_ms = (self.config.max_vel_time * 1000.0) as i64;

            // Out-of-order timestamps: don't apply velocity checks to movement that predates the velocity.
            if vel_age_ms >= 0 && vel_age_ms < max_vel_ms {
                // Expected horizontal velocity effect
                let expected_h = (vx * vx + vz * vz).sqrt();
                
                // Check if player is taking the velocity
                if expected_h > 0.1 && h_dist < expected_h * 0.3 {
                    // Player ignored significant knockback
                    let ignored_ratio = 1.0 - (h_dist / expected_h);
                    let mitigated = state.movement.distance_vl.update(ignored_ratio as f32, timestamp_ms);

                    if ignored_ratio > 0.5 {
                        findings.push(
                            Finding::new(
                                state.player_uuid,
                                FeatureId::AacMoveVel,
                                ignored_ratio as f32,
                                state.movement.distance_vl.get(),
                                mitigated,
                                timestamp_ms,
                            )
                            .with_description(format!(
                                "Ignored {:.0}% of velocity (expected {:.2}, got {:.2})",
                                ignored_ratio * 100.0,
                                expected_h,
                                h_dist
                            )),
                        );
                    }
                }
            } else {
                // Velocity expired
                state.movement.pending_velocity = None;
            }
        }

        findings
    }

    fn check_noslow_sneak(
        &self,
        state: &mut PlayerState,
        h_dist: f64,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        if h_dist > MAX_SNEAK_SPEED {
            let ratio = h_dist / MAX_SNEAK_SPEED;
            let mitigated = state.movement.distance_vl.update((ratio - 1.0) as f32, timestamp_ms);

            if ratio > 1.3 {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AacMoveNoslow,
                        h_dist as f32,
                        state.movement.distance_vl.get(),
                        mitigated,
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Moving {:.2} b/t while sneaking (max {:.2})",
                        h_dist, MAX_SNEAK_SPEED
                    )),
                );
            }
        }

        findings
    }

    fn check_noslow_item(
        &self,
        state: &mut PlayerState,
        h_dist: f64,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Item use slows to ~20% speed
        let max_speed = MAX_WALK_SPEED * 0.2;
        if h_dist > max_speed {
            let ratio = h_dist / max_speed;
            let mitigated = state.movement.distance_vl.update((ratio - 1.0) as f32, timestamp_ms);

            if ratio > 1.5 {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AacMoveNoslow,
                        h_dist as f32,
                        state.movement.distance_vl.get(),
                        mitigated,
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Moving {:.2} b/t while using item (max {:.2})",
                        h_dist, max_speed
                    )),
                );
            }
        }

        findings
    }

    fn check_generic_movement(
        &self,
        state: &mut PlayerState,
        h_dist: f64,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Allow higher speeds if velocity is pending
        let max_speed = if state.movement.pending_velocity.is_some() {
            MAX_SPRINT_SPEED * 3.0 // Allow velocity boost
        } else {
            MAX_SPRINT_SPEED
        };

        if h_dist > max_speed {
            // Use the same max_speed for ratio calculation to avoid inflated values
            let ratio = h_dist / max_speed;
            let mitigated = state.movement.distance_vl.update((ratio - 1.0) as f32, timestamp_ms);

            if ratio > 1.5 {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::AacMoveGeneric,
                        h_dist as f32,
                        state.movement.distance_vl.get(),
                        mitigated,
                        timestamp_ms,
                    )
                    .with_description(format!(
                        "Horizontal speed {:.2} b/t (max {:.2})",
                        h_dist, max_speed
                    )),
                );
            }
        }

        findings
    }

    fn handle_velocity(
        &self,
        state: &mut PlayerState,
        vel: &EntityVelocityPacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        // Only store velocity if it's for this player
        // entity_id 0 is invalid, and we need to match the player's entity ID
        // Since we don't track entity IDs per player, we use a heuristic:
        // - entity_id < 0 is likely invalid
        // - Very high entity IDs are suspicious
        // For now, we'll accept entity_id > 0 as potentially valid
        // A proper fix would track player entity IDs from spawn packets
        if vel.entity_id > 0 {
            state.movement.pending_velocity = Some((vel.velocity_x, vel.velocity_y, vel.velocity_z));
            state.movement.velocity_received_ms = timestamp_ms;
        }
        Vec::new()
    }

    fn handle_sneak(&self, state: &mut PlayerState, sneak: &SneakPacket) {
        state.movement.is_sneaking = sneak.sneaking;
    }
}

