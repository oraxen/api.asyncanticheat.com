//! Player state for combat module

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

use crate::buffer::{CheckBuffer, SampleBuffer};
use crate::packets::Location;

/// Complete player state for combat checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub player_uuid: Uuid,
    pub killaura: KillAuraState,
    pub aim: AimState,
    pub autoclicker: AutoClickerState,
    pub reach: ReachState,
    pub noswing: NoSwingState,
    pub combat: CombatState,
    pub movement: MovementState,
}

impl PlayerState {
    pub fn new(player_uuid: Uuid) -> Self {
        Self {
            player_uuid,
            killaura: KillAuraState::default(),
            aim: AimState::default(),
            autoclicker: AutoClickerState::default(),
            reach: ReachState::default(),
            noswing: NoSwingState::default(),
            combat: CombatState::default(),
            movement: MovementState::default(),
        }
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new(Uuid::nil())
    }
}

/// KillAura detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillAuraState {
    pub last_attack_ms: i64,
    pub last_target_id: Option<i32>,
    pub target_switches: u32,
    pub rapid_attacks: u32,
    pub buffer: CheckBuffer,
}

impl Default for KillAuraState {
    fn default() -> Self {
        Self {
            last_attack_ms: 0,
            last_target_id: None,
            target_switches: 0,
            rapid_attacks: 0,
            buffer: CheckBuffer::new(2.0, 10, 0.95),
        }
    }
}

/// Aim detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AimState {
    pub last_yaw: f32,
    pub last_pitch: f32,
    pub last_rotation_ms: i64,
    pub yaw_deltas: SampleBuffer,
    pub pitch_deltas: SampleBuffer,
    pub yaw_history: VecDeque<f32>,
    pub pitch_history: VecDeque<f32>,
    pub sensitivity_samples: SampleBuffer,
    pub snap_count: u32,
    pub last_snap_ms: i64,
    pub buffer_headsnap: CheckBuffer,
    pub buffer_pitch: CheckBuffer,
    pub buffer_sens: CheckBuffer,
    pub buffer_modulo: CheckBuffer,
    pub buffer_dirswitch: CheckBuffer,
    pub buffer_repeated: CheckBuffer,
}

impl Default for AimState {
    fn default() -> Self {
        Self {
            last_yaw: 0.0,
            last_pitch: 0.0,
            last_rotation_ms: 0,
            yaw_deltas: SampleBuffer::new(20),
            pitch_deltas: SampleBuffer::new(20),
            yaw_history: VecDeque::with_capacity(20),
            pitch_history: VecDeque::with_capacity(20),
            sensitivity_samples: SampleBuffer::new(10),
            snap_count: 0,
            last_snap_ms: 0,
            buffer_headsnap: CheckBuffer::new(3.0, 10, 0.9),
            buffer_pitch: CheckBuffer::new(2.0, 10, 0.95),
            buffer_sens: CheckBuffer::new(2.0, 10, 0.95),
            buffer_modulo: CheckBuffer::new(3.0, 10, 0.9),
            buffer_dirswitch: CheckBuffer::new(2.0, 10, 0.9),
            buffer_repeated: CheckBuffer::new(3.0, 10, 0.9),
        }
    }
}

/// AutoClicker detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoClickerState {
    pub last_click_ms: i64,
    pub click_intervals: SampleBuffer,
    pub clicks_in_window: u32,
    pub window_start_ms: i64,
    pub last_stats: ClickStats,
    pub buffer_cps: CheckBuffer,
    pub buffer_timing: CheckBuffer,
    pub buffer_variance: CheckBuffer,
    pub buffer_kurtosis: CheckBuffer,
    pub buffer_tickalign: CheckBuffer,
}

impl Default for AutoClickerState {
    fn default() -> Self {
        Self {
            last_click_ms: 0,
            click_intervals: SampleBuffer::new(20),
            clicks_in_window: 0,
            window_start_ms: 0,
            last_stats: ClickStats::default(),
            buffer_cps: CheckBuffer::new(1.0, 10, 0.9),
            buffer_timing: CheckBuffer::new(2.0, 10, 0.95),
            buffer_variance: CheckBuffer::new(2.0, 10, 0.95),
            buffer_kurtosis: CheckBuffer::new(2.0, 10, 0.95),
            buffer_tickalign: CheckBuffer::new(3.0, 10, 0.9),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClickStats {
    pub cps: f64,
    pub std_dev: f64,
    pub variance: f64,
    pub kurtosis: f64,
    pub distinct: usize,
}

/// Reach detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReachState {
    pub last_reach: f64,
    pub reach_samples: SampleBuffer,
    pub buffer: CheckBuffer,
    pub vl: f64,
}

impl Default for ReachState {
    fn default() -> Self {
        Self {
            last_reach: 0.0,
            reach_samples: SampleBuffer::new(10),
            buffer: CheckBuffer::new(1.0, 5, 0.8),
            vl: 0.0,
        }
    }
}

/// NoSwing detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoSwingState {
    pub last_swing_ms: i64,
    pub attacks_without_swing: u32,
    pub vl: f64,
    pub buffer: CheckBuffer,
}

impl Default for NoSwingState {
    fn default() -> Self {
        Self {
            last_swing_ms: 0,
            attacks_without_swing: 0,
            vl: 0.0,
            buffer: CheckBuffer::new(3.0, 5, 0.9),
        }
    }
}

/// General combat state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    pub in_combat: bool,
    pub combat_ticks: u32,
    pub last_combat_ms: i64,
    pub total_attacks: u32,
}

impl Default for CombatState {
    fn default() -> Self {
        Self {
            in_combat: false,
            combat_ticks: 0,
            last_combat_ms: 0,
            total_attacks: 0,
        }
    }
}

/// Movement state (for position tracking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementState {
    pub last_location: Option<Location>,
    pub last_move_ms: i64,
}

impl Default for MovementState {
    fn default() -> Self {
        Self {
            last_location: None,
            last_move_ms: 0,
        }
    }
}
