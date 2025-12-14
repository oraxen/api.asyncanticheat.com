use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;
use uuid::Uuid;

// =============================================================================
// Shared types (module internal)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub player_uuid: Option<Uuid>,
    pub detector_name: String,
    pub detector_version: Option<String>,
    pub severity: String,
    pub title: String,
    pub description: Option<String>,
    pub evidence_s3_key: Option<String>,
    pub evidence_json: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    #[serde(default)]
    pub version: u32,

    #[serde(default)]
    pub fight_angle: FightAngleState,

    #[serde(default)]
    pub fight_speed: FightSpeedState,

    #[serde(default)]
    pub fight_reach: FightReachState,

    #[serde(default)]
    pub fight_direction: FightDirectionState,

    #[serde(default)]
    pub fight_wrongturn: FightWrongTurnState,

    #[serde(default)]
    pub moving_basic: MovingBasicState,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            version: 1,
            fight_angle: FightAngleState::default(),
            fight_speed: FightSpeedState::default(),
            fight_reach: FightReachState::default(),
            fight_direction: FightDirectionState::default(),
            fight_wrongturn: FightWrongTurnState::default(),
            moving_basic: MovingBasicState::default(),
        }
    }
}

// =============================================================================
// Combat (NCP-inspired): Angle + AttackSpeed
// =============================================================================

#[derive(Debug, Clone)]
pub struct FightConfig {
    pub angle_threshold: f64,
    pub angle_max_window_ms: u64,
    pub speed_limit_aps: f64,
    pub speed_window_ms: u64,
    pub reach_limit_blocks: f64,
    pub direction_off_threshold: f64,
}

impl Default for FightConfig {
    fn default() -> Self {
        Self {
            // NCP default is config-driven; pick conservative defaults.
            angle_threshold: 50.0,
            angle_max_window_ms: 1000,
            speed_limit_aps: 8.0,
            speed_window_ms: 1000,
            // NCP default reach in survival is ~4.4 (configurable).
            reach_limit_blocks: 4.4,
            // NCP uses off > 0.1 for strict-ish checks.
            direction_off_threshold: 0.1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FightAngleState {
    #[serde(default)]
    pub vl: f64,
    /// Recent attacks in a 1s window (newest last).
    #[serde(default)]
    pub recent: VecDeque<AttackSample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackSample {
    pub ts: u64,
    pub target_entity_id: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FightSpeedState {
    #[serde(default)]
    pub vl: f64,
    /// Attack timestamps in a sliding window (newest last).
    #[serde(default)]
    pub recent_attack_ts: VecDeque<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FightReachState {
    #[serde(default)]
    pub vl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FightDirectionState {
    #[serde(default)]
    pub vl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FightWrongTurnState {
    #[serde(default)]
    pub vl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MovingBasicState {
    #[serde(default)]
    pub speed_vl: f64,
    #[serde(default)]
    pub nofall_vl: f64,
    #[serde(default)]
    pub more_packets_vl: f64,
}

#[derive(Debug, Clone)]
pub struct CombatEvent {
    pub ts: u64,
    pub player_uuid: Uuid,
    pub entity_id: i64,
    pub player_x: Option<f64>,
    pub player_y: Option<f64>,
    pub player_z: Option<f64>,
    pub player_yaw: Option<f64>,
    pub player_pitch: Option<f64>,
    pub dt_ms: Option<f64>,
    pub target_switched: Option<bool>,
    pub yaw_diff: Option<f64>,
    pub reach_distance: Option<f64>,
    pub aim_off: Option<f64>,
}

pub fn process_combat_events(
    cfg: &FightConfig,
    state: &mut PlayerState,
    player_uuid: Uuid,
    events: &[CombatEvent],
    evidence_s3_key: Option<&str>,
) -> Vec<Finding> {
    let mut out = Vec::new();
    // Events are expected sorted by ts (transform emits sequentially).
    for ev in events {
        if ev.player_uuid != player_uuid {
            continue;
        }

        // --- WrongTurn (NCP-inspired): invalid pitch (> 90 deg) ---
        if let Some(pitch) = ev.player_pitch {
            if pitch.abs() > 90.0 {
                state.fight_wrongturn.vl += 1.0; // NCP never cools down.
                out.push(Finding {
                    player_uuid: Some(player_uuid),
                    detector_name: "ncp_fight_wrongturn".to_string(),
                    detector_version: Some("1".to_string()),
                    severity: "violation".to_string(),
                    title: "Invalid pitch (wrong turn)".to_string(),
                    description: Some(format!("pitch={:.2} vl={:.0}", pitch, state.fight_wrongturn.vl)),
                    evidence_s3_key: evidence_s3_key.map(|s| s.to_string()),
                    evidence_json: Some(serde_json::json!({
                        "pitch": pitch,
                        "vl": state.fight_wrongturn.vl
                    })),
                });
            }
        }

        // --- FightSpeed (NCP-inspired): attacks per second in a sliding window ---
        {
            let window_ms = cfg.speed_window_ms.max(1);
            state.fight_speed.recent_attack_ts.push_back(ev.ts);
            while let Some(front) = state.fight_speed.recent_attack_ts.front().copied() {
                if ev.ts.saturating_sub(front) > window_ms {
                    state.fight_speed.recent_attack_ts.pop_front();
                } else {
                    break;
                }
            }

            let elapsed_ms = {
                let first = state.fight_speed.recent_attack_ts.front().copied().unwrap_or(ev.ts);
                ev.ts.saturating_sub(first)
            } as f64;
            let count = state.fight_speed.recent_attack_ts.len() as f64;

            // APS is based on intervals between attacks; the first attack should not instantly flag.
            let aps = if count <= 1.0 || elapsed_ms <= 0.0 {
                0.0
            } else {
                let intervals = count - 1.0;
                intervals * 1000.0 / elapsed_ms
            };

            if aps > cfg.speed_limit_aps {
                let added = aps - cfg.speed_limit_aps;
                state.fight_speed.vl += added;
                out.push(Finding {
                    player_uuid: Some(player_uuid),
                    detector_name: "ncp_fight_speed".to_string(),
                    detector_version: Some("1".to_string()),
                    severity: if state.fight_speed.vl >= 10.0 {
                        "violation".to_string()
                    } else {
                        "warning".to_string()
                    },
                    title: "Unusually fast attacks".to_string(),
                    description: Some(format!(
                        "aps={:.2} limit={:.2} vl={:.2}",
                        aps, cfg.speed_limit_aps, state.fight_speed.vl
                    )),
                    evidence_s3_key: evidence_s3_key.map(|s| s.to_string()),
                    evidence_json: Some(serde_json::json!({
                        "aps": aps,
                        "limit_aps": cfg.speed_limit_aps,
                        "vl": state.fight_speed.vl,
                        "window_ms": cfg.speed_window_ms
                    })),
                });
            } else {
                // NCP decays ~0.96 on pass.
                state.fight_speed.vl *= 0.96;
            }
        }

        // --- FightReach (NCP-inspired, simplified): reach distance above limit ---
        if let Some(dist) = ev.reach_distance {
            let violation = dist - cfg.reach_limit_blocks;
            if violation > 0.0 {
                state.fight_reach.vl += violation;
                out.push(Finding {
                    player_uuid: Some(player_uuid),
                    detector_name: "ncp_fight_reach".to_string(),
                    detector_version: Some("1".to_string()),
                    severity: if state.fight_reach.vl >= 5.0 {
                        "violation".to_string()
                    } else {
                        "warning".to_string()
                    },
                    title: "Attack reach too far".to_string(),
                    description: Some(format!(
                        "reach={:.3} limit={:.2} vl={:.2}",
                        dist, cfg.reach_limit_blocks, state.fight_reach.vl
                    )),
                    evidence_s3_key: evidence_s3_key.map(|s| s.to_string()),
                    evidence_json: Some(serde_json::json!({
                        "reach_distance": dist,
                        "limit_blocks": cfg.reach_limit_blocks,
                        "vl": state.fight_reach.vl
                    })),
                });
            } else {
                state.fight_reach.vl *= 0.8;
            }
        }

        // --- FightDirection (NCP-inspired, simplified): aim off distance above threshold ---
        if let Some(off) = ev.aim_off {
            if off > cfg.direction_off_threshold {
                state.fight_direction.vl += off;
                out.push(Finding {
                    player_uuid: Some(player_uuid),
                    detector_name: "ncp_fight_direction".to_string(),
                    detector_version: Some("1".to_string()),
                    severity: if state.fight_direction.vl >= 2.0 {
                        "violation".to_string()
                    } else {
                        "warning".to_string()
                    },
                    title: "Attack outside field-of-view".to_string(),
                    description: Some(format!(
                        "aim_off={:.3} threshold={:.2} vl={:.2}",
                        off, cfg.direction_off_threshold, state.fight_direction.vl
                    )),
                    evidence_s3_key: evidence_s3_key.map(|s| s.to_string()),
                    evidence_json: Some(serde_json::json!({
                        "aim_off": off,
                        "threshold": cfg.direction_off_threshold,
                        "vl": state.fight_direction.vl
                    })),
                });
            } else {
                state.fight_direction.vl *= 0.8;
            }
        }

        // --- FightAngle (NCP-inspired): rapid target switching with large yaw changes ---
        // Requires position + yaw context; if missing, skip to keep signal clean.
        let (Some(x), Some(y), Some(z), Some(yaw)) = (ev.player_x, ev.player_y, ev.player_z, ev.player_yaw) else {
            continue;
        };

        let window_ms = cfg.angle_max_window_ms.max(1);
        state.fight_angle.recent.push_back(AttackSample {
            ts: ev.ts,
            target_entity_id: ev.entity_id,
            x,
            y,
            z,
            yaw,
        });
        while let Some(front) = state.fight_angle.recent.front().cloned() {
            if ev.ts.saturating_sub(front.ts) > window_ms {
                state.fight_angle.recent.pop_front();
            } else {
                break;
            }
        }

        if state.fight_angle.recent.len() < 2 {
            continue;
        }

        // Recompute the NCP-style aggregates over the window.
        let mut delta_move = 0.0f64; // sum distSqLast
        let mut delta_time = 0.0f64; // sum timeDiff (ms)
        let mut delta_yaw = 0.0f64; // sum yawDiff
        let mut delta_switch = 0.0f64; // count switches (float for averaging)

        let mut prev: Option<&AttackSample> = None;
        for s in state.fight_angle.recent.iter() {
            if let Some(p) = prev {
                let dx = s.x - p.x;
                let dy = s.y - p.y;
                let dz = s.z - p.z;
                delta_move += dx * dx + dy * dy + dz * dz;

                let dt = (s.ts.saturating_sub(p.ts)) as f64;
                delta_time += dt;

                let yd = yaw_difference(s.yaw, p.yaw);
                delta_yaw += yd;

                if s.target_entity_id != p.target_entity_id && yd > 30.0 {
                    delta_switch += 1.0;
                }
            }
            prev = Some(s);
        }

        let n = (state.fight_angle.recent.len() - 1) as f64;
        let average_move = delta_move / n;
        let average_time = delta_time / n;
        let average_yaw = delta_yaw / n;
        let average_switching = delta_switch / n;

        // NCP-inspired violation formula (Angle.java).
        let mut violation = 0.0f64;
        if (0.0..0.2).contains(&average_move) {
            violation += 20.0 * (0.2 - average_move) / 0.2;
        }
        if (0.0..150.0).contains(&average_time) {
            violation += 30.0 * (150.0 - average_time) / 150.0;
        }
        if average_yaw > 50.0 {
            violation += 30.0 * average_yaw / 180.0;
        }
        if average_switching > 0.0 {
            violation += 20.0 * average_switching;
        }

        if violation > cfg.angle_threshold {
            state.fight_angle.vl += violation;
            out.push(Finding {
                player_uuid: Some(player_uuid),
                detector_name: "ncp_fight_angle".to_string(),
                detector_version: Some("1".to_string()),
                severity: if state.fight_angle.vl >= 100.0 {
                    "violation".to_string()
                } else {
                    "warning".to_string()
                },
                title: "Suspicious attack angle pattern".to_string(),
                description: Some(format!(
                    "v={:.2} vl={:.2} avg_time_ms={:.1} avg_yaw={:.1} avg_switching={:.2}",
                    violation, state.fight_angle.vl, average_time, average_yaw, average_switching
                )),
                evidence_s3_key: evidence_s3_key.map(|s| s.to_string()),
                evidence_json: Some(serde_json::json!({
                    "violation": violation,
                    "vl": state.fight_angle.vl,
                    "avg_move_sq": average_move,
                    "avg_time_ms": average_time,
                    "avg_yaw_deg": average_yaw,
                    "avg_switching": average_switching,
                    "threshold": cfg.angle_threshold,
                    "window_ms": cfg.angle_max_window_ms
                })),
            });
        } else {
            // NCP: data.angleVL *= 0.98
            state.fight_angle.vl *= 0.98;
        }
    }
    out
}

// =============================================================================
// Movement (lightweight, NCP-inspired heuristics)
// =============================================================================

#[derive(Debug, Clone)]
pub struct MovingConfig {
    pub speed_limit_bps: f64,
    pub more_packets_dt_ms: f64,
    pub nofall_dy_threshold: f64,
}

impl Default for MovingConfig {
    fn default() -> Self {
        Self {
            speed_limit_bps: 15.0,
            more_packets_dt_ms: 5.0,
            nofall_dy_threshold: -3.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MovementEvent {
    pub ts: u64,
    pub player_uuid: Uuid,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub dt_ms: Option<f64>,
    pub dx: Option<f64>,
    pub dy: Option<f64>,
    pub dz: Option<f64>,
    pub speed_bps: Option<f64>,
    pub on_ground: Option<bool>,
}

pub fn process_movement_events(
    cfg: &MovingConfig,
    state: &mut PlayerState,
    player_uuid: Uuid,
    events: &[MovementEvent],
    evidence_s3_key: Option<&str>,
) -> Vec<Finding> {
    let mut out = Vec::new();
    for ev in events {
        if ev.player_uuid != player_uuid {
            continue;
        }

        // Basic speed check using transform-provided speed_bps.
        if let Some(speed) = ev.speed_bps {
            if speed > cfg.speed_limit_bps {
                let added = speed - cfg.speed_limit_bps;
                state.moving_basic.speed_vl += added;
                out.push(Finding {
                    player_uuid: Some(player_uuid),
                    detector_name: "ncp_moving_speed_basic".to_string(),
                    detector_version: Some("1".to_string()),
                    severity: if state.moving_basic.speed_vl >= 20.0 {
                        "violation".to_string()
                    } else {
                        "warning".to_string()
                    },
                    title: "Unusually high movement speed".to_string(),
                    description: Some(format!(
                        "speed_bps={:.2} limit={:.2} vl={:.2}",
                        speed, cfg.speed_limit_bps, state.moving_basic.speed_vl
                    )),
                    evidence_s3_key: evidence_s3_key.map(|s| s.to_string()),
                    evidence_json: Some(serde_json::json!({
                        "speed_bps": speed,
                        "limit_bps": cfg.speed_limit_bps,
                        "vl": state.moving_basic.speed_vl,
                        "pos": {"x": ev.x, "y": ev.y, "z": ev.z},
                        "d": {"dt_ms": ev.dt_ms, "dx": ev.dx, "dy": ev.dy, "dz": ev.dz}
                    })),
                });
            } else {
                state.moving_basic.speed_vl *= 0.98;
            }
        }

        // MorePackets heuristic: extremely low dt_ms repeatedly.
        if let Some(dt) = ev.dt_ms {
            if dt > 0.0 && dt < cfg.more_packets_dt_ms {
                state.moving_basic.more_packets_vl += (cfg.more_packets_dt_ms - dt) / cfg.more_packets_dt_ms;
                out.push(Finding {
                    player_uuid: Some(player_uuid),
                    detector_name: "ncp_moving_morepackets_basic".to_string(),
                    detector_version: Some("1".to_string()),
                    severity: if state.moving_basic.more_packets_vl >= 10.0 {
                        "violation".to_string()
                    } else {
                        "warning".to_string()
                    },
                    title: "Unusually frequent movement packets".to_string(),
                    description: Some(format!(
                        "dt_ms={:.2} threshold<{:.2} vl={:.2}",
                        dt, cfg.more_packets_dt_ms, state.moving_basic.more_packets_vl
                    )),
                    evidence_s3_key: evidence_s3_key.map(|s| s.to_string()),
                    evidence_json: Some(serde_json::json!({
                        "dt_ms": dt,
                        "threshold_ms": cfg.more_packets_dt_ms,
                        "vl": state.moving_basic.more_packets_vl
                    })),
                });
            } else {
                state.moving_basic.more_packets_vl *= 0.98;
            }
        }

        // NoFall heuristic: large negative dy while claiming on_ground=true.
        if let (Some(dy), Some(on_ground)) = (ev.dy, ev.on_ground) {
            if dy < cfg.nofall_dy_threshold && on_ground {
                state.moving_basic.nofall_vl += (-dy).min(10.0);
                out.push(Finding {
                    player_uuid: Some(player_uuid),
                    detector_name: "ncp_moving_nofall_basic".to_string(),
                    detector_version: Some("1".to_string()),
                    severity: if state.moving_basic.nofall_vl >= 50.0 {
                        "violation".to_string()
                    } else {
                        "warning".to_string()
                    },
                    title: "Suspicious on_ground during fall".to_string(),
                    description: Some(format!(
                        "dy={:.3} on_ground=true vl={:.2}",
                        dy, state.moving_basic.nofall_vl
                    )),
                    evidence_s3_key: evidence_s3_key.map(|s| s.to_string()),
                    evidence_json: Some(serde_json::json!({
                        "dy": dy,
                        "threshold_dy": cfg.nofall_dy_threshold,
                        "on_ground": on_ground,
                        "vl": state.moving_basic.nofall_vl
                    })),
                });
            } else {
                state.moving_basic.nofall_vl *= 0.98;
            }
        }
    }
    out
}

// =============================================================================
// Helpers
// =============================================================================

/// Absolute yaw difference in degrees with wraparound at 360.
pub fn yaw_difference(yaw1: f64, yaw2: f64) -> f64 {
    let mut diff = (yaw1 - yaw2).abs();
    if diff > 180.0 {
        diff = 360.0 - diff;
    }
    diff
}


