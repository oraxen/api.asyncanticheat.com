//! AAC5 Configuration with defaults from config.yml
//!
//! All time units are in seconds, all length units are in blocks.

use serde::{Deserialize, Serialize};

/// VL (Violation Level) accumulator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VlConfig {
    /// Whether to actively mitigate/block when threshold is crossed
    pub mitigate: bool,
    /// VL threshold at/above which mitigation applies
    pub threshold: f32,
    /// Maximum VL cap (forced to at least 1.0)
    pub max: f32,
    /// VL decay per second
    pub decay: f32,
}

impl Default for VlConfig {
    fn default() -> Self {
        Self {
            mitigate: true,
            threshold: 1.0,
            max: 1.0,
            decay: 0.01,
        }
    }
}

/// Delays check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelaysConfig {
    pub enabled: bool,
    pub vl: VlConfig,
    /// Fast block breaking
    pub fast_break: bool,
    /// Fast block placing
    pub fast_place: bool,
    /// Fast item use (eating, drinking)
    pub fast_use: bool,
    /// Fast bow shooting
    pub fast_bow: bool,
    /// Fast regeneration
    pub regen: bool,
    /// Fast sneak toggle
    pub fast_sneak: bool,
    /// Fast bow release
    pub fast_release: bool,
    /// Break delay violations
    pub break_delay: bool,
}

impl Default for DelaysConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            vl: VlConfig::default(),
            fast_break: true,
            fast_place: true,
            fast_use: true,
            fast_bow: true,
            regen: true,
            fast_sneak: true,
            fast_release: true,
            break_delay: true,
        }
    }
}

/// Move check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveConfig {
    pub enabled: bool,
    /// Distance VL accumulator
    pub vl: VlConfig,
    /// Timer VL accumulator (client running faster)
    pub timer: VlConfig,
    /// Phasing severity multiplier
    pub phase_vl: f32,
    /// Block NoFall damage avoidance
    pub block_nofall: bool,
    /// Velocity acceptance window (seconds)
    pub max_vel_time: f32,
    /// Piston memory duration (seconds)
    pub piston_wait_time: f32,
    /// Check sneak speed
    pub check_sneak: bool,
    /// Check item use speed
    pub check_item_use: bool,
    /// Check flying players
    pub check_flying: bool,
    // Fallback thresholds (blocks/tick)
    pub flowing_speed: f32,
    pub bubble_column_speed: f32,
    pub bumping_speed: f32,
    pub elytra_rocket_speed: f32,
    pub riptide_speed: f32,
    pub boat_hitbox_tolerance: f32,
    pub shulker_hitbox_tolerance: f32,
    pub elytra_landing_tolerance: f32,
}

impl Default for MoveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            vl: VlConfig::default(),
            timer: VlConfig::default(),
            phase_vl: 1.0,
            block_nofall: true,
            max_vel_time: 1.0,
            piston_wait_time: 1.0,
            check_sneak: true,
            check_item_use: true,
            check_flying: false,
            flowing_speed: 0.2,
            bubble_column_speed: 1.8,
            bumping_speed: 0.1,
            elytra_rocket_speed: 2.0,
            riptide_speed: 4.2,
            boat_hitbox_tolerance: 0.8,
            shulker_hitbox_tolerance: 0.6,
            elytra_landing_tolerance: 0.5,
        }
    }
}

/// Hardcoded movement constants from AAC5 ca_0
#[derive(Debug, Clone, Copy)]
pub struct MoveConstants {
    /// R = 0.03
    pub r: f64,
    /// u = 9.0E-4
    pub u: f64,
    /// t = 0.12
    pub t: f64,
    /// o = 0.0144
    pub o: f64,
    /// aa = 0.001
    pub aa: f64,
    /// J = 1.0E-6
    pub j: f64,
}

impl Default for MoveConstants {
    fn default() -> Self {
        Self {
            r: 0.03,
            u: 9.0E-4,
            t: 0.12,
            o: 0.0144,
            aa: 0.001,
            j: 1.0E-6,
        }
    }
}

/// Aimbot check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AimbotConfig {
    pub enabled: bool,
    pub check_sensitivity: bool,
    pub check_mouse_delta: bool,
    pub check_head_snap: bool,
    pub check_pitch_spread: bool,
    pub check_zero_point: bool,
    /// Analysis window size (fixed at 50 in AAC5)
    pub sample_size: usize,
}

impl Default for AimbotConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_sensitivity: true,
            check_mouse_delta: true,
            check_head_snap: true,
            check_pitch_spread: true,
            check_zero_point: true,
            sample_size: 50, // static int s = 50 in c7
        }
    }
}

/// Autoclicker check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoclickerConfig {
    pub enabled: bool,
    pub check_tick_delay: bool,
    pub check_noswing: bool,
    pub check_timing: bool,
    /// Sample buffer size
    pub sample_size: usize,
}

impl Default for AutoclickerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_tick_delay: true,
            check_noswing: true,
            check_timing: true,
            sample_size: 20, // checks.autoclicker.sample_size: 20
        }
    }
}

/// Hitbox check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitboxConfig {
    pub enabled: bool,
    pub vl: VlConfig,
    /// Lag compensation ticks
    pub lag_compensation_ticks: u32,
    /// Hit queue size
    pub hit_queue_size: usize,
    /// Check block occlusion
    pub check_blocks: bool,
    /// Base reach constant (static double n = 3.0 in ch_0)
    pub base_reach: f64,
}

impl Default for HitboxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            vl: VlConfig {
                mitigate: true,
                threshold: 0.5,
                max: 1.0,
                decay: 0.01,
            },
            lag_compensation_ticks: 20,
            hit_queue_size: 4,
            check_blocks: true,
            base_reach: 3.0, // static double n = 3.0 in ch_0
        }
    }
}

/// Interact check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractConfig {
    pub enabled: bool,
    pub vl: VlConfig,
    /// Maximum allowed angle difference (radians)
    pub max_angle_diff: f32,
    /// Exempt materials
    pub exempt: Vec<String>,
}

impl Default for InteractConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            vl: VlConfig::default(),
            max_angle_diff: 1.0, // radians
            exempt: Vec::new(),
        }
    }
}

/// Misc check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiscConfig {
    pub enabled: bool,
    pub invalid_pitch: bool,
    pub player_abilities: bool,
    pub rotation_rate: bool,
}

impl Default for MiscConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            invalid_pitch: true,
            player_abilities: true,
            rotation_rate: true,
        }
    }
}

/// Misc check constants from cO and cC
#[derive(Debug, Clone, Copy)]
pub struct MiscConstants {
    /// Large sentinel for rotation math (static float j = 1.0E8f in cO)
    pub rotation_sentinel: f32,
    /// Short tick window (g = 20L in cC, 1 second at 20 TPS)
    pub short_window_ticks: u64,
    /// Medium tick window (a = 200L in cC, 10 seconds)
    pub medium_window_ticks: u64,
    /// Long tick window (l = 1200L in cC, 60 seconds)
    pub long_window_ticks: u64,
    /// Time threshold for rotation branches (100L ms)
    pub rotation_time_threshold_ms: u64,
}

impl Default for MiscConstants {
    fn default() -> Self {
        Self {
            rotation_sentinel: 1.0E8,
            short_window_ticks: 20,
            medium_window_ticks: 200,
            long_window_ticks: 1200,
            rotation_time_threshold_ms: 100,
        }
    }
}

/// Complete AAC configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AacConfig {
    pub delays: DelaysConfig,
    pub r#move: MoveConfig,
    pub aimbot: AimbotConfig,
    pub autoclicker: AutoclickerConfig,
    pub hitbox: HitboxConfig,
    pub interact: InteractConfig,
    pub misc: MiscConfig,
}

impl Default for AacConfig {
    fn default() -> Self {
        Self {
            delays: DelaysConfig::default(),
            r#move: MoveConfig::default(),
            aimbot: AimbotConfig::default(),
            autoclicker: AutoclickerConfig::default(),
            hitbox: HitboxConfig::default(),
            interact: InteractConfig::default(),
            misc: MiscConfig::default(),
        }
    }
}

