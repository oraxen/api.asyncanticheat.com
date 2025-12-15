//! Player module configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub badpackets: BadPacketsConfig,
    pub scaffold: ScaffoldConfig,
    pub fastplace: FastPlaceConfig,
    pub fastbreak: FastBreakConfig,
    pub interact: InteractConfig,
    pub inventory: InventoryConfig,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            badpackets: BadPacketsConfig::default(),
            scaffold: ScaffoldConfig::default(),
            fastplace: FastPlaceConfig::default(),
            fastbreak: FastBreakConfig::default(),
            interact: InteractConfig::default(),
            inventory: InventoryConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadPacketsConfig {
    pub enabled: bool,
    /// Maximum valid pitch angle (vanilla max is 90)
    pub max_pitch: f32,
    /// Maximum hotbar slot index (0-8)
    pub max_hotbar_slot: i32,
    /// Maximum flying packets per second before flagging
    pub max_flying_packets_per_sec: u32,
    /// Check for NaN positions
    pub check_nan_position: bool,
    /// Check for abilities spoofing
    pub check_abilities_spoof: bool,
}

impl Default for BadPacketsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_pitch: 90.0,
            max_hotbar_slot: 8,
            max_flying_packets_per_sec: 25,
            check_nan_position: true,
            check_abilities_spoof: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldConfig {
    pub enabled: bool,
    /// Check for placing on bottom face while airborne
    pub check_airborne_bottom: bool,
    /// Check for sprinting while bridging
    pub check_sprint_bridge: bool,
    /// Minimum consecutive scaffold placements before flagging
    pub min_scaffold_count: u32,
}

impl Default for ScaffoldConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_airborne_bottom: true,
            check_sprint_bridge: true,
            min_scaffold_count: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastPlaceConfig {
    pub enabled: bool,
    /// Minimum milliseconds between block placements
    pub min_place_interval_ms: i64,
    /// Critical threshold for immediate flag
    pub critical_interval_ms: i64,
}

impl Default for FastPlaceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_place_interval_ms: 50,
            critical_interval_ms: 25,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastBreakConfig {
    pub enabled: bool,
    /// Minimum milliseconds between block breaks
    pub min_break_interval_ms: i64,
    /// Critical threshold for immediate flag
    pub critical_interval_ms: i64,
}

impl Default for FastBreakConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_break_interval_ms: 50,
            critical_interval_ms: 25,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractConfig {
    pub enabled: bool,
    /// Maximum valid interaction pitch deviation from look direction
    pub max_angle_deviation: f32,
    /// Check for impossible interaction angles
    pub check_impossible_angles: bool,
}

impl Default for InteractConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_angle_deviation: 45.0,
            check_impossible_angles: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryConfig {
    pub enabled: bool,
    /// Minimum milliseconds between inventory clicks
    pub fastclick_window_ms: i64,
    /// Number of fast clicks before flagging
    pub fast_click_threshold: u32,
}

impl Default for InventoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fastclick_window_ms: 200,
            fast_click_threshold: 5,
        }
    }
}
