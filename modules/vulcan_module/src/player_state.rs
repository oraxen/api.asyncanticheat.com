//! Per-player state tracking for Vulcan checks

use std::collections::VecDeque;
use uuid::Uuid;

use crate::buffer::CheckBuffer;
use crate::config::VulcanConfig;
use crate::packets::Location;

/// Statistical sample buffer
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

    pub fn skewness(&self) -> f64 {
        if self.samples.len() < 3 {
            return 0.0;
        }
        let mean = self.mean();
        let std_dev = self.std_dev();
        if std_dev == 0.0 {
            return 0.0;
        }
        let n = self.samples.len() as f64;
        let sum_cubed: f64 = self.samples.iter().map(|x| ((x - mean) / std_dev).powi(3)).sum();
        (n / ((n - 1.0) * (n - 2.0))) * sum_cubed
    }

    pub fn kurtosis(&self) -> f64 {
        if self.samples.len() < 4 {
            return 0.0;
        }
        let mean = self.mean();
        let std_dev = self.std_dev();
        if std_dev == 0.0 {
            return 0.0;
        }
        let n = self.samples.len() as f64;
        let sum_fourth: f64 = self.samples.iter().map(|x| ((x - mean) / std_dev).powi(4)).sum();
        let term1 = (n * (n + 1.0)) / ((n - 1.0) * (n - 2.0) * (n - 3.0));
        let term2 = (3.0 * (n - 1.0).powi(2)) / ((n - 2.0) * (n - 3.0));
        term1 * sum_fourth - term2
    }

    pub fn distinct_count(&self) -> usize {
        let mut distinct: Vec<i64> = self.samples.iter()
            .map(|&x| (x * 1000.0) as i64) // Convert to fixed-point for comparison
            .collect();
        distinct.sort();
        distinct.dedup();
        distinct.len()
    }

    pub fn range(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let min = self.samples.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = self.samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        max - min
    }

    pub fn outlier_count(&self, threshold: f64) -> usize {
        let mean = self.mean();
        let std_dev = self.std_dev();
        if std_dev == 0.0 {
            return 0;
        }
        self.samples
            .iter()
            .filter(|&&x| ((x - mean) / std_dev).abs() > threshold)
            .count()
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, f64> {
        self.samples.iter()
    }

    pub fn last_n(&self, n: usize) -> Vec<f64> {
        let len = self.samples.len();
        if n >= len {
            self.samples.iter().cloned().collect()
        } else {
            self.samples.iter().skip(len - n).cloned().collect()
        }
    }
}

/// Combat state tracking
#[derive(Debug, Clone)]
pub struct CombatState {
    pub in_combat: bool,
    pub combat_ticks: u32,
    pub last_attack_ms: i64,
    pub last_attack_target: Option<i32>,
    pub last_swing_ms: i64,
    pub attacks_since_swing: u32,
    pub rapid_attack: bool,
}

impl Default for CombatState {
    fn default() -> Self {
        Self {
            in_combat: false,
            combat_ticks: 0,
            last_attack_ms: 0,
            last_attack_target: None,
            last_swing_ms: 0,
            attacks_since_swing: 0,
            rapid_attack: false,
        }
    }
}

/// Aim check state
#[derive(Debug, Clone)]
pub struct AimState {
    pub last_yaw: f32,
    pub last_pitch: f32,
    pub last_rotation_ms: i64,
    pub yaw_deltas: SampleBuffer,
    pub pitch_deltas: SampleBuffer,
    pub yaw_history: VecDeque<f32>,
    pub pitch_history: VecDeque<f32>,
    // Buffers for each aim check
    pub buffer_a: CheckBuffer,
    pub buffer_b: CheckBuffer,
    pub buffer_c: CheckBuffer,
    pub buffer_l: CheckBuffer,
}

impl AimState {
    pub fn new(config: &VulcanConfig) -> Self {
        Self {
            last_yaw: 0.0,
            last_pitch: 0.0,
            last_rotation_ms: 0,
            yaw_deltas: SampleBuffer::new(50),
            pitch_deltas: SampleBuffer::new(50),
            yaw_history: VecDeque::with_capacity(20),
            pitch_history: VecDeque::with_capacity(20),
            buffer_a: CheckBuffer::new(config.aim.a.buffer.clone(), config.aim.a.max_vl),
            buffer_b: CheckBuffer::new(config.aim.b.buffer.clone(), config.aim.b.max_vl),
            buffer_c: CheckBuffer::new(config.aim.c.buffer.clone(), config.aim.c.max_vl),
            buffer_l: CheckBuffer::new(config.aim.l.buffer.clone(), config.aim.l.max_vl),
        }
    }
}

/// Auto Clicker state
#[derive(Debug, Clone)]
pub struct AutoClickerState {
    pub click_intervals: SampleBuffer,
    pub last_click_ms: i64,
    pub clicks_in_window: u32,
    pub window_start_ms: i64,
    pub last_stats: ClickStats,
    // Buffers
    pub buffer_a: CheckBuffer,
    pub buffer_b: CheckBuffer,
    pub buffer_e: CheckBuffer,
    pub buffer_i: CheckBuffer,
}

#[derive(Debug, Clone, Default)]
pub struct ClickStats {
    pub cps: f64,
    pub std_dev: f64,
    pub variance: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub distinct: usize,
}

impl AutoClickerState {
    pub fn new(config: &VulcanConfig) -> Self {
        Self {
            click_intervals: SampleBuffer::new(config.autoclicker.sample_size),
            last_click_ms: 0,
            clicks_in_window: 0,
            window_start_ms: 0,
            last_stats: ClickStats::default(),
            buffer_a: CheckBuffer::autoclicker(),
            buffer_b: CheckBuffer::autoclicker(),
            buffer_e: CheckBuffer::autoclicker(),
            buffer_i: CheckBuffer::autoclicker(),
        }
    }
}

/// Velocity state
#[derive(Debug, Clone)]
pub struct VelocityState {
    pub pending_velocity: Option<(f64, f64, f64)>,
    pub velocity_received_ms: i64,
    pub ticks_since_velocity: u32,
    pub buffer_a: CheckBuffer,
    pub buffer_b: CheckBuffer,
}

impl VelocityState {
    pub fn new(config: &VulcanConfig) -> Self {
        Self {
            pending_velocity: None,
            velocity_received_ms: 0,
            ticks_since_velocity: 0,
            buffer_a: CheckBuffer::new(config.velocity.a.buffer.clone(), config.velocity.a.max_vl),
            buffer_b: CheckBuffer::new(config.velocity.b.buffer.clone(), config.velocity.b.max_vl),
        }
    }
}

/// Movement state
#[derive(Debug, Clone)]
pub struct MovementState {
    pub last_location: Option<Location>,
    pub last_on_ground: bool,
    pub last_move_ms: i64,
    pub motion_y: f64,
    pub last_motion_y: f64,
    // Prediction
    pub predicted_y: f64,
    // Timer
    pub packet_timestamps: SampleBuffer,
    pub tick_count: u32,
    // States
    pub is_sprinting: bool,
    pub is_sneaking: bool,
    pub is_blocking: bool,
    pub in_liquid: bool,
    pub on_ladder: bool,
    pub using_elytra: bool,
    pub in_vehicle: bool,
    // Flying packet count
    pub flying_packet_count: u32,
    // Buffers
    pub flight_buffer: CheckBuffer,
    pub speed_buffer: CheckBuffer,
    pub timer_buffer: CheckBuffer,
    pub groundspoof_buffer: CheckBuffer,
}

impl MovementState {
    pub fn new(config: &VulcanConfig) -> Self {
        Self {
            last_location: None,
            last_on_ground: true,
            last_move_ms: 0,
            motion_y: 0.0,
            last_motion_y: 0.0,
            predicted_y: 0.0,
            packet_timestamps: SampleBuffer::new(config.timer.sample_size),
            tick_count: 0,
            is_sprinting: false,
            is_sneaking: false,
            is_blocking: false,
            in_liquid: false,
            on_ladder: false,
            using_elytra: false,
            in_vehicle: false,
            flying_packet_count: 0,
            flight_buffer: CheckBuffer::flight(),
            speed_buffer: CheckBuffer::speed(),
            timer_buffer: CheckBuffer::new(config.timer.a.buffer.clone(), config.timer.a.max_vl),
            groundspoof_buffer: CheckBuffer::new(config.groundspoof.a.buffer.clone(), config.groundspoof.a.max_vl),
        }
    }
}

/// Bad Packets state
#[derive(Debug, Clone)]
pub struct BadPacketsState {
    pub last_packet_type: String,
    pub last_packet_ms: i64,
    pub last_hotbar_slot: i32,
    pub last_keep_alive_id: i64,
    pub pending_keep_alives: VecDeque<i64>,
}

impl Default for BadPacketsState {
    fn default() -> Self {
        Self {
            last_packet_type: String::new(),
            last_packet_ms: 0,
            last_hotbar_slot: 0,
            last_keep_alive_id: 0,
            pending_keep_alives: VecDeque::with_capacity(10),
        }
    }
}

/// Scaffold state
#[derive(Debug, Clone)]
pub struct ScaffoldState {
    pub last_place_ms: i64,
    pub last_place_face: String,
    pub place_count: u32,
    pub buffer: CheckBuffer,
}

impl ScaffoldState {
    pub fn new(config: &VulcanConfig) -> Self {
        Self {
            last_place_ms: 0,
            last_place_face: String::new(),
            place_count: 0,
            buffer: CheckBuffer::new(config.scaffold.a.buffer.clone(), config.scaffold.a.max_vl),
        }
    }
}

/// Complete per-player state
#[derive(Debug, Clone)]
pub struct PlayerState {
    pub player_uuid: Uuid,
    pub player_name: String,
    pub combat: CombatState,
    pub aim: AimState,
    pub autoclicker: AutoClickerState,
    pub velocity: VelocityState,
    pub movement: MovementState,
    pub badpackets: BadPacketsState,
    pub scaffold: ScaffoldState,
    pub last_update_ms: i64,
    pub session_start_ms: i64,
    pub ticks_existed: u32,
    // Improbable (meta-check) counters
    pub combat_violations: u32,
    pub movement_violations: u32,
    pub player_violations: u32,
}

impl PlayerState {
    pub fn new(player_uuid: Uuid, player_name: String, config: &VulcanConfig, timestamp_ms: i64) -> Self {
        Self {
            player_uuid,
            player_name,
            combat: CombatState::default(),
            aim: AimState::new(config),
            autoclicker: AutoClickerState::new(config),
            velocity: VelocityState::new(config),
            movement: MovementState::new(config),
            badpackets: BadPacketsState::default(),
            scaffold: ScaffoldState::new(config),
            last_update_ms: timestamp_ms,
            session_start_ms: timestamp_ms,
            ticks_existed: 0,
            combat_violations: 0,
            movement_violations: 0,
            player_violations: 0,
        }
    }

    pub fn touch(&mut self, timestamp_ms: i64) {
        self.last_update_ms = timestamp_ms;
    }

    pub fn is_stale(&self, current_ms: i64, timeout_ms: i64) -> bool {
        current_ms - self.last_update_ms > timeout_ms
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "player_uuid": self.player_uuid.to_string(),
            "player_name": self.player_name,
            "ticks_existed": self.ticks_existed,
            "combat": {
                "in_combat": self.combat.in_combat,
                "combat_ticks": self.combat.combat_ticks,
            },
            "aim": {
                "buffer_a": self.aim.buffer_a.get(),
                "buffer_b": self.aim.buffer_b.get(),
                "vl_a": self.aim.buffer_a.vl(),
                "vl_b": self.aim.buffer_b.vl(),
            },
            "autoclicker": {
                "cps": self.autoclicker.last_stats.cps,
                "std_dev": self.autoclicker.last_stats.std_dev,
                "buffer_a": self.autoclicker.buffer_a.get(),
            },
            "velocity": {
                "pending": self.velocity.pending_velocity.is_some(),
                "buffer_a": self.velocity.buffer_a.get(),
            },
            "movement": {
                "flight_buffer": self.movement.flight_buffer.get(),
                "speed_buffer": self.movement.speed_buffer.get(),
                "timer_buffer": self.movement.timer_buffer.get(),
            },
            "improbable": {
                "combat_vl": self.combat_violations,
                "movement_vl": self.movement_violations,
                "player_vl": self.player_violations,
            },
        })
    }
}

