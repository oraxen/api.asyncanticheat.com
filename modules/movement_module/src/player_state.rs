//! Player state for movement module

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

use crate::buffer::{CheckBuffer, SampleBuffer};
use crate::packets::Location;

/// Complete player state for movement checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub player_uuid: Uuid,
    pub flight: FlightState,
    pub speed: SpeedState,
    pub nofall: NoFallState,
    pub timer: TimerState,
    pub step: StepState,
    pub groundspoof: GroundSpoofState,
    pub velocity: VelocityState,
    pub noslow: NoSlowState,
    pub movement: MovementState,
}

impl PlayerState {
    pub fn new(player_uuid: Uuid) -> Self {
        Self {
            player_uuid,
            flight: FlightState::default(),
            speed: SpeedState::default(),
            nofall: NoFallState::default(),
            timer: TimerState::default(),
            step: StepState::default(),
            groundspoof: GroundSpoofState::default(),
            velocity: VelocityState::default(),
            noslow: NoSlowState::default(),
            movement: MovementState::default(),
        }
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new(Uuid::nil())
    }
}

/// Flight detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightState {
    pub last_y: f64,
    pub last_y_delta: f64,
    pub predicted_y_delta: f64,
    pub ascend_ticks: u32,
    pub hover_ticks: u32,
    pub air_ticks: u32,
    pub last_on_ground_y: f64,
    pub buffer_ypred: CheckBuffer,
    pub buffer_ascend: CheckBuffer,
    pub buffer_hover: CheckBuffer,
}

impl Default for FlightState {
    fn default() -> Self {
        Self {
            last_y: 0.0,
            last_y_delta: 0.0,
            predicted_y_delta: 0.0,
            ascend_ticks: 0,
            hover_ticks: 0,
            air_ticks: 0,
            last_on_ground_y: 0.0,
            buffer_ypred: CheckBuffer::new(2.0, 10, 0.9),
            buffer_ascend: CheckBuffer::new(2.0, 10, 0.9),
            buffer_hover: CheckBuffer::new(2.0, 10, 0.9),
        }
    }
}

/// Speed detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedState {
    pub is_sprinting: bool,
    pub is_sneaking: bool,
    pub speed_samples: SampleBuffer,
    pub buffer_horizontal: CheckBuffer,
    pub buffer_sprint: CheckBuffer,
    pub buffer_sneak: CheckBuffer,
}

impl Default for SpeedState {
    fn default() -> Self {
        Self {
            is_sprinting: false,
            is_sneaking: false,
            speed_samples: SampleBuffer::new(20),
            buffer_horizontal: CheckBuffer::new(2.0, 10, 0.9),
            buffer_sprint: CheckBuffer::new(2.0, 10, 0.9),
            buffer_sneak: CheckBuffer::new(2.0, 10, 0.9),
        }
    }
}

/// NoFall detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoFallState {
    pub fall_distance: f64,
    pub last_y_velocity: f64,
    pub invalid_ground_claims: u32,
    pub last_damage_y: f64,
    pub buffer: CheckBuffer,
}

impl Default for NoFallState {
    fn default() -> Self {
        Self {
            fall_distance: 0.0,
            last_y_velocity: 0.0,
            invalid_ground_claims: 0,
            last_damage_y: 0.0,
            buffer: CheckBuffer::new(2.0, 10, 0.9),
        }
    }
}

/// Timer detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub packet_timestamps: VecDeque<i64>,
    pub last_packet_ms: i64,
    pub packets_in_window: u32,
    pub window_start_ms: i64,
    pub balance_ms: f64,
    pub buffer_fast: CheckBuffer,
    pub buffer_slow: CheckBuffer,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            packet_timestamps: VecDeque::with_capacity(100),
            last_packet_ms: 0,
            packets_in_window: 0,
            window_start_ms: 0,
            balance_ms: 0.0,
            buffer_fast: CheckBuffer::new(2.0, 10, 0.9),
            buffer_slow: CheckBuffer::new(3.0, 10, 0.95),
        }
    }
}

/// Step detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepState {
    pub last_y: f64,
    pub was_on_ground: bool,
    pub buffer: CheckBuffer,
}

impl Default for StepState {
    fn default() -> Self {
        Self {
            last_y: 0.0,
            was_on_ground: false,
            buffer: CheckBuffer::new(1.0, 5, 0.8),
        }
    }
}

/// Ground spoof detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundSpoofState {
    pub y_velocity: f64,
    pub consecutive_spoofs: u32,
    pub last_ground_y: f64,
    pub buffer: CheckBuffer,
}

impl Default for GroundSpoofState {
    fn default() -> Self {
        Self {
            y_velocity: 0.0,
            consecutive_spoofs: 0,
            last_ground_y: 0.0,
            buffer: CheckBuffer::new(2.0, 8, 0.9),
        }
    }
}

/// Velocity detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityState {
    pub pending_velocity: Option<PendingVelocity>,
    pub ignored_count: u32,
    pub last_velocity_ms: i64,
    pub buffer: CheckBuffer,
}

impl Default for VelocityState {
    fn default() -> Self {
        Self {
            pending_velocity: None,
            ignored_count: 0,
            last_velocity_ms: 0,
            buffer: CheckBuffer::new(2.0, 10, 0.9),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingVelocity {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub timestamp_ms: i64,
    pub ticks_elapsed: u32,
}

/// NoSlow detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoSlowState {
    pub is_using_item: bool,
    pub use_item_start_ms: i64,
    pub buffer_item: CheckBuffer,
    pub buffer_sneak: CheckBuffer,
}

impl Default for NoSlowState {
    fn default() -> Self {
        Self {
            is_using_item: false,
            use_item_start_ms: 0,
            buffer_item: CheckBuffer::new(2.0, 10, 0.9),
            buffer_sneak: CheckBuffer::new(2.0, 10, 0.9),
        }
    }
}

/// General movement state (for position tracking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementState {
    pub current_location: Option<Location>,
    pub last_location: Option<Location>,
    pub last_move_ms: i64,
    pub on_ground: bool,
    pub was_on_ground: bool,
    pub ticks_since_teleport: u32,
}

impl Default for MovementState {
    fn default() -> Self {
        Self {
            current_location: None,
            last_location: None,
            last_move_ms: 0,
            on_ground: false,
            was_on_ground: false,
            ticks_since_teleport: 100,
        }
    }
}
