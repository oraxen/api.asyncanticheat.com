//! Per-player state tracking for AAC checks
//!
//! Based on me.konsolas.aac.cz_0 (per-player check manager)

use std::collections::VecDeque;
use uuid::Uuid;

use crate::config::{AacConfig, MiscConstants, MoveConstants};
use crate::packets::Location;
use crate::vl::ViolationLevel;

/// Ring buffer for storing samples
#[derive(Debug, Clone)]
pub struct SampleBuffer {
    samples: VecDeque<f64>,
    capacity: usize,
}

impl SampleBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, value: f64) {
        if self.samples.len() >= self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(value);
    }

    pub fn is_full(&self) -> bool {
        self.samples.len() >= self.capacity
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn mean(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        self.samples.iter().sum::<f64>() / self.samples.len() as f64
    }

    pub fn variance(&self) -> f64 {
        if self.samples.len() < 2 {
            return 0.0;
        }
        let mean = self.mean();
        let sum_sq: f64 = self.samples.iter().map(|x| (x - mean).powi(2)).sum();
        sum_sq / (self.samples.len() - 1) as f64
    }

    pub fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }

    pub fn iter(&self) -> impl Iterator<Item = &f64> {
        self.samples.iter()
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

/// Delays check state (cm_0)
#[derive(Debug, Clone)]
pub struct DelaysState {
    pub vl: ViolationLevel,
    // Timestamps for various delay checks (r, q, m, n, h, u, f in cm_0)
    pub last_break_start_ms: i64,
    pub last_break_end_ms: i64,
    pub last_place_ms: i64,
    pub last_use_ms: i64,
    pub last_bow_ms: i64,
    pub last_regen_ms: i64,
    pub last_sneak_ms: i64,
    pub last_release_ms: i64,
    // Block being broken
    pub breaking_block: Option<(i32, i32, i32)>,
}

impl DelaysState {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            vl: ViolationLevel::new(config.delays.vl.clone()),
            last_break_start_ms: 0,
            last_break_end_ms: 0,
            last_place_ms: 0,
            last_use_ms: 0,
            last_bow_ms: 0,
            last_regen_ms: 0,
            last_sneak_ms: 0,
            last_release_ms: 0,
            breaking_block: None,
        }
    }
}

/// Move check state (ca_0)
#[derive(Debug, Clone)]
pub struct MoveState {
    pub distance_vl: ViolationLevel,
    pub timer_vl: ViolationLevel,
    // Movement state
    pub last_location: Option<Location>,
    pub last_on_ground: bool,
    pub last_move_ms: i64,
    // Velocity tracking
    pub pending_velocity: Option<(f64, f64, f64)>,
    pub velocity_received_ms: i64,
    // Timer tracking
    pub move_count: u32,
    pub timer_start_ms: i64,
    // Special states
    pub in_vehicle: bool,
    pub using_elytra: bool,
    pub in_water: bool,
    pub in_lava: bool,
    pub is_sneaking: bool,
    pub using_item: bool,
    // Fall tracking
    pub fall_distance: f64,
    pub last_ground_y: f64,
    // Movement constants
    pub constants: MoveConstants,
}

impl MoveState {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            distance_vl: ViolationLevel::new(config.r#move.vl.clone()),
            timer_vl: ViolationLevel::new(config.r#move.timer.clone()),
            last_location: None,
            last_on_ground: true,
            last_move_ms: 0,
            pending_velocity: None,
            velocity_received_ms: 0,
            move_count: 0,
            timer_start_ms: 0,
            in_vehicle: false,
            using_elytra: false,
            in_water: false,
            in_lava: false,
            is_sneaking: false,
            using_item: false,
            fall_distance: 0.0,
            last_ground_y: 0.0,
            constants: MoveConstants::default(),
        }
    }
}

/// Aimbot check state (c7)
#[derive(Debug, Clone)]
pub struct AimbotState {
    // Rotation tracking
    pub last_yaw: f32,
    pub last_pitch: f32,
    pub last_rotation_ms: i64,
    // Delta tracking
    pub yaw_deltas: SampleBuffer,
    pub pitch_deltas: SampleBuffer,
    // Sensitivity analysis
    pub sensitivity_samples: SampleBuffer,
    // Head snap detection
    pub last_snap_ms: i64,
    pub snap_count: u32,
}

impl AimbotState {
    pub fn new(config: &AacConfig) -> Self {
        let sample_size = config.aimbot.sample_size;
        Self {
            last_yaw: 0.0,
            last_pitch: 0.0,
            last_rotation_ms: 0,
            yaw_deltas: SampleBuffer::new(sample_size),
            pitch_deltas: SampleBuffer::new(sample_size),
            sensitivity_samples: SampleBuffer::new(sample_size),
            last_snap_ms: 0,
            snap_count: 0,
        }
    }
}

/// Autoclicker check state (cr_0)
#[derive(Debug, Clone)]
pub struct AutoclickerState {
    // Click timing samples
    pub click_intervals: SampleBuffer,
    pub last_click_ms: i64,
    // Swing tracking
    pub last_swing_ms: i64,
    pub attacks_without_swing: u32,
    // CPS tracking
    pub clicks_in_window: u32,
    pub window_start_ms: i64,
}

impl AutoclickerState {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            click_intervals: SampleBuffer::new(config.autoclicker.sample_size),
            last_click_ms: 0,
            last_swing_ms: 0,
            attacks_without_swing: 0,
            clicks_in_window: 0,
            window_start_ms: 0,
        }
    }
}

/// Hitbox check state (ch_0)
#[derive(Debug, Clone)]
pub struct HitboxState {
    pub vl: ViolationLevel,
    // Attack tracking (note: we only see attack attempts, not confirmed hits)
    pub attacks: u32,
    // Reach samples
    pub reach_samples: VecDeque<f64>,
    pub last_attack_ms: i64,
    // Target tracking
    pub last_target_id: Option<i32>,
    pub last_target_distance: f64,
}

impl HitboxState {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            vl: ViolationLevel::new(config.hitbox.vl.clone()),
            attacks: 0,
            reach_samples: VecDeque::with_capacity(10),
            last_attack_ms: 0,
            last_target_id: None,
            last_target_distance: 0.0,
        }
    }
}

/// Interact check state (ce_0)
#[derive(Debug, Clone)]
pub struct InteractState {
    pub vl: ViolationLevel,
    pub last_interact_ms: i64,
    pub invalid_interactions: u32,
}

impl InteractState {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            vl: ViolationLevel::new(config.interact.vl.clone()),
            last_interact_ms: 0,
            invalid_interactions: 0,
        }
    }
}

/// Misc check state (cO + cC)
#[derive(Debug, Clone)]
pub struct MiscState {
    // Rotation tracking (cO)
    pub last_yaw: f32,
    pub last_pitch: f32,
    pub last_rotation_ms: i64,
    // Rate counters (cC)
    pub short_window_count: u32,
    pub medium_window_count: u32,
    pub long_window_count: u32,
    pub short_window_start_ms: i64,
    pub medium_window_start_ms: i64,
    pub long_window_start_ms: i64,
    // Flags
    pub invalid_pitch_flagged: bool,
    pub invalid_abilities_flagged: bool,
    // Constants
    pub constants: MiscConstants,
}

impl MiscState {
    pub fn new(_config: &AacConfig) -> Self {
        Self {
            last_yaw: 0.0,
            last_pitch: 0.0,
            last_rotation_ms: 0,
            short_window_count: 0,
            medium_window_count: 0,
            long_window_count: 0,
            short_window_start_ms: 0,
            medium_window_start_ms: 0,
            long_window_start_ms: 0,
            invalid_pitch_flagged: false,
            invalid_abilities_flagged: false,
            constants: MiscConstants::default(),
        }
    }
}

/// Complete per-player state
#[derive(Debug, Clone)]
pub struct PlayerState {
    pub player_uuid: Uuid,
    pub player_name: String,
    pub delays: DelaysState,
    pub movement: MoveState,
    pub aimbot: AimbotState,
    pub autoclicker: AutoclickerState,
    pub hitbox: HitboxState,
    pub interact: InteractState,
    pub misc: MiscState,
    /// Last update timestamp
    pub last_update_ms: i64,
    /// Session start time
    pub session_start_ms: i64,
}

impl PlayerState {
    pub fn new(player_uuid: Uuid, player_name: String, config: &AacConfig, timestamp_ms: i64) -> Self {
        Self {
            player_uuid,
            player_name,
            delays: DelaysState::new(config),
            movement: MoveState::new(config),
            aimbot: AimbotState::new(config),
            autoclicker: AutoclickerState::new(config),
            hitbox: HitboxState::new(config),
            interact: InteractState::new(config),
            misc: MiscState::new(config),
            last_update_ms: timestamp_ms,
            session_start_ms: timestamp_ms,
        }
    }

    /// Update the last activity timestamp
    pub fn touch(&mut self, timestamp_ms: i64) {
        self.last_update_ms = timestamp_ms;
    }

    /// Check if this player state is stale (no activity for too long)
    pub fn is_stale(&self, current_ms: i64, timeout_ms: i64) -> bool {
        current_ms - self.last_update_ms > timeout_ms
    }

    /// Update VL configurations when runtime config changes
    /// This ensures existing players get the new threshold/decay/mitigation settings
    pub fn update_config(&mut self, config: &AacConfig) {
        self.delays.vl.update_config(config.delays.vl.clone());
        self.movement.distance_vl.update_config(config.r#move.vl.clone());
        self.movement.timer_vl.update_config(config.r#move.timer.clone());
        self.hitbox.vl.update_config(config.hitbox.vl.clone());
        self.interact.vl.update_config(config.interact.vl.clone());
    }

    /// Serialize state for persistence
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "player_uuid": self.player_uuid.to_string(),
            "player_name": self.player_name,
            "delays_vl": self.delays.vl.get(),
            "move_distance_vl": self.movement.distance_vl.get(),
            "move_timer_vl": self.movement.timer_vl.get(),
            "hitbox_vl": self.hitbox.vl.get(),
            "interact_vl": self.interact.vl.get(),
            "last_update_ms": self.last_update_ms,
            "session_start_ms": self.session_start_ms,
            "aimbot": {
                "yaw_deltas_count": self.aimbot.yaw_deltas.len(),
                "pitch_deltas_count": self.aimbot.pitch_deltas.len(),
                "snap_count": self.aimbot.snap_count,
            },
            "autoclicker": {
                "click_intervals_count": self.autoclicker.click_intervals.len(),
                "attacks_without_swing": self.autoclicker.attacks_without_swing,
            },
            "hitbox": {
                "attacks": self.hitbox.attacks,
            },
        })
    }
}

