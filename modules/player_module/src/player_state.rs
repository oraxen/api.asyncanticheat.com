//! Player state for player module

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::buffer::{CheckBuffer, SampleBuffer};
use crate::packets::Location;

/// Complete player state for player checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub player_uuid: Uuid,
    pub badpackets: BadPacketsState,
    pub scaffold: ScaffoldState,
    pub fastplace: FastPlaceState,
    pub fastbreak: FastBreakState,
    pub interact: InteractState,
    pub inventory: InventoryState,
    pub movement: MovementState,
}

impl PlayerState {
    pub fn new(player_uuid: Uuid) -> Self {
        Self {
            player_uuid,
            badpackets: BadPacketsState::default(),
            scaffold: ScaffoldState::default(),
            fastplace: FastPlaceState::default(),
            fastbreak: FastBreakState::default(),
            interact: InteractState::default(),
            inventory: InventoryState::default(),
            movement: MovementState::default(),
        }
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new(Uuid::nil())
    }
}

/// BadPackets detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadPacketsState {
    pub flying_packets_this_sec: u32,
    pub flying_window_start_ms: i64,
    pub last_abilities_flying: bool,
    pub server_allows_flight: bool,
    /// Whether the server allows instant block breaking (creative mode)
    pub server_allows_instant_break: bool,
    pub buffer_pitch: CheckBuffer,
    pub buffer_nan: CheckBuffer,
    /// Buffer for flying abilities spoofing detection
    pub buffer_abilities: CheckBuffer,
    /// Buffer for instant_break abilities spoofing detection (separate from flying)
    pub buffer_instant_break: CheckBuffer,
    pub buffer_slot: CheckBuffer,
    pub buffer_flying_flood: CheckBuffer,
}

impl Default for BadPacketsState {
    fn default() -> Self {
        Self {
            flying_packets_this_sec: 0,
            flying_window_start_ms: 0,
            last_abilities_flying: false,
            server_allows_flight: false,
            server_allows_instant_break: false,
            buffer_pitch: CheckBuffer::new(1.0, 5, 0.9),
            buffer_nan: CheckBuffer::new(1.0, 3, 0.9),
            buffer_abilities: CheckBuffer::new(2.0, 5, 0.9),
            buffer_instant_break: CheckBuffer::new(2.0, 5, 0.9),
            buffer_slot: CheckBuffer::new(1.0, 5, 0.9),
            buffer_flying_flood: CheckBuffer::new(3.0, 10, 0.95),
        }
    }
}

/// Scaffold detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldState {
    pub consecutive_scaffold: u32,
    pub last_place_ms: i64,
    pub last_place_face: i32,
    pub is_sprinting: bool,
    pub buffer_airborne: CheckBuffer,
    pub buffer_sprint: CheckBuffer,
}

impl Default for ScaffoldState {
    fn default() -> Self {
        Self {
            consecutive_scaffold: 0,
            last_place_ms: 0,
            last_place_face: -1,
            is_sprinting: false,
            buffer_airborne: CheckBuffer::new(2.0, 10, 0.9),
            buffer_sprint: CheckBuffer::new(2.0, 10, 0.9),
        }
    }
}

/// FastPlace detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastPlaceState {
    pub last_place_ms: i64,
    pub place_intervals: SampleBuffer,
    pub fast_place_count: u32,
    pub buffer: CheckBuffer,
    pub buffer_critical: CheckBuffer,
}

impl Default for FastPlaceState {
    fn default() -> Self {
        Self {
            last_place_ms: 0,
            place_intervals: SampleBuffer::new(20),
            fast_place_count: 0,
            buffer: CheckBuffer::new(2.0, 10, 0.95),
            buffer_critical: CheckBuffer::new(1.0, 5, 0.9),
        }
    }
}

/// FastBreak detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastBreakState {
    pub last_break_ms: i64,
    pub break_intervals: SampleBuffer,
    pub fast_break_count: u32,
    pub buffer: CheckBuffer,
    pub buffer_critical: CheckBuffer,
}

impl Default for FastBreakState {
    fn default() -> Self {
        Self {
            last_break_ms: 0,
            break_intervals: SampleBuffer::new(20),
            fast_break_count: 0,
            buffer: CheckBuffer::new(2.0, 10, 0.95),
            buffer_critical: CheckBuffer::new(1.0, 5, 0.9),
        }
    }
}

/// Interact detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractState {
    pub last_interact_yaw: f32,
    pub last_interact_pitch: f32,
    pub buffer_angle: CheckBuffer,
    pub buffer_impossible: CheckBuffer,
}

impl Default for InteractState {
    fn default() -> Self {
        Self {
            last_interact_yaw: 0.0,
            last_interact_pitch: 0.0,
            buffer_angle: CheckBuffer::new(2.0, 10, 0.95),
            buffer_impossible: CheckBuffer::new(1.0, 5, 0.9),
        }
    }
}

/// Inventory detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryState {
    pub last_click_ms: i64,
    pub click_intervals: SampleBuffer,
    pub fast_clicks_count: u32,
    pub buffer: CheckBuffer,
}

impl Default for InventoryState {
    fn default() -> Self {
        Self {
            last_click_ms: 0,
            click_intervals: SampleBuffer::new(20),
            fast_clicks_count: 0,
            buffer: CheckBuffer::new(3.0, 10, 0.95),
        }
    }
}

/// Movement state (for position tracking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementState {
    pub last_location: Option<Location>,
    pub last_move_ms: i64,
    pub on_ground: bool,
    pub last_yaw: f32,
    pub last_pitch: f32,
}

impl Default for MovementState {
    fn default() -> Self {
        Self {
            last_location: None,
            last_move_ms: 0,
            on_ground: true,
            last_yaw: 0.0,
            last_pitch: 0.0,
        }
    }
}
