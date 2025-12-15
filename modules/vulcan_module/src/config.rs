//! Vulcan Configuration with defaults from config.yml
//!
//! All check configurations follow the pattern:
//! - enabled: whether the check runs
//! - punishable: whether it can punish at max VL
//! - max_vl: maximum violations before punishment
//! - buffer: BufferConfig with max, multiple, decay

use serde::{Deserialize, Serialize};
use crate::buffer::BufferConfig;

/// Individual check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckConfig {
    pub enabled: bool,
    pub punishable: bool,
    pub max_vl: u32,
    pub alert_interval: u32,
    pub dont_alert_until: u32,
    pub buffer: BufferConfig,
    pub max_ping: u32,
    pub min_tps: f64,
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            punishable: true,
            max_vl: 10,
            alert_interval: 1,
            dont_alert_until: 1,
            buffer: BufferConfig::default(),
            max_ping: 100000,
            min_tps: -1.0,
        }
    }
}

// ============================================================================
// Combat Check Configs
// ============================================================================

/// Aim check configurations (A through Y)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AimConfig {
    pub a: CheckConfig, // Slope - Invalid pitch change
    pub b: CheckConfig, // Modulo - Invalid yaw change
    pub c: CheckConfig, // Repeated - Repeated yaw values
    pub d: CheckConfig, // Straight - Invalid pitch (non-exp)
    pub e: CheckConfig, // Ratio - Invalid yaw (non-exp)
    pub f: CheckConfig, // Straight - Invalid yaw
    pub g: CheckConfig, // Ratio - Too large yaw
    pub h: CheckConfig, // Negative - Invalid sensitivity
    pub i: CheckConfig, // Constant - Not constant rotations
    pub k: CheckConfig, // Linear - Not constant rotations
    pub l: CheckConfig, // Direction - Switching too quickly
    pub m: CheckConfig, // Small Yaw
    pub n: CheckConfig, // Small Yaw (non-exp)
    pub o: CheckConfig, // Small Pitch
    pub p: CheckConfig, // Yaw Acceleration
    pub q: CheckConfig, // GCD Modulo
    pub r: CheckConfig, // Analysis
    pub s: CheckConfig, // Divisor Y
    pub u: CheckConfig, // GCD Flaw
    pub w: CheckConfig, // Analysis
    pub x: CheckConfig, // Analysis
    pub y: CheckConfig, // Rotation
}

impl Default for AimConfig {
    fn default() -> Self {
        Self {
            a: CheckConfig {
                max_vl: 1,
                buffer: BufferConfig { max: 3.0, multiple: 0.75, decay: 0.5 },
                ..Default::default()
            },
            b: CheckConfig {
                max_vl: 8,
                buffer: BufferConfig { max: 6.0, multiple: 0.5, decay: 0.65 },
                ..Default::default()
            },
            c: CheckConfig {
                max_vl: 5,
                buffer: BufferConfig { max: 5.0, multiple: 0.5, decay: 0.75 },
                ..Default::default()
            },
            d: CheckConfig {
                max_vl: 10,
                buffer: BufferConfig { max: 8.0, multiple: 0.5, decay: 0.75 },
                ..Default::default()
            },
            e: CheckConfig {
                max_vl: 3,
                buffer: BufferConfig { max: 4.0, multiple: 0.75, decay: 0.25 },
                ..Default::default()
            },
            f: CheckConfig {
                max_vl: 10,
                buffer: BufferConfig { max: 12.0, multiple: 0.5, decay: 0.45 },
                ..Default::default()
            },
            g: CheckConfig {
                max_vl: 5,
                buffer: BufferConfig { max: 4.0, multiple: 0.5, decay: 0.75 },
                ..Default::default()
            },
            h: CheckConfig {
                max_vl: 10,
                buffer: BufferConfig { max: 7.0, multiple: 0.5, decay: 0.75 },
                ..Default::default()
            },
            i: CheckConfig {
                max_vl: 10,
                buffer: BufferConfig { max: 7.0, multiple: 0.5, decay: 0.75 },
                ..Default::default()
            },
            k: CheckConfig {
                max_vl: 15,
                buffer: BufferConfig { max: 6.0, multiple: 0.5, decay: 0.25 },
                ..Default::default()
            },
            l: CheckConfig {
                max_vl: 5,
                buffer: BufferConfig { max: 10.0, multiple: 0.45, decay: 0.0 },
                ..Default::default()
            },
            m: CheckConfig {
                max_vl: 3,
                buffer: BufferConfig { max: 2.0, multiple: 0.4, decay: 0.25 },
                ..Default::default()
            },
            n: CheckConfig {
                max_vl: 10,
                buffer: BufferConfig { max: 5.0, multiple: 0.5, decay: 0.5 },
                ..Default::default()
            },
            o: CheckConfig {
                max_vl: 10,
                buffer: BufferConfig { max: 5.0, multiple: 0.5, decay: 0.25 },
                ..Default::default()
            },
            p: CheckConfig {
                max_vl: 5,
                buffer: BufferConfig { max: 2.0, multiple: 0.4, decay: 0.25 },
                ..Default::default()
            },
            q: CheckConfig {
                max_vl: 5,
                buffer: BufferConfig { max: 6.0, multiple: 0.5, decay: 0.25 },
                ..Default::default()
            },
            r: CheckConfig {
                max_vl: 5,
                buffer: BufferConfig { max: 20.0, multiple: 0.3, decay: 0.5 },
                ..Default::default()
            },
            s: CheckConfig {
                max_vl: 5,
                buffer: BufferConfig { max: 20.0, multiple: 0.25, decay: 0.5 },
                ..Default::default()
            },
            u: CheckConfig {
                max_vl: 1,
                buffer: BufferConfig { max: 0.0, multiple: 0.0, decay: 0.0 },
                ..Default::default()
            },
            w: CheckConfig {
                max_vl: 5,
                buffer: BufferConfig { max: 20.0, multiple: 0.25, decay: 0.125 },
                ..Default::default()
            },
            x: CheckConfig {
                max_vl: 5,
                buffer: BufferConfig { max: 12.0, multiple: 0.25, decay: 0.125 },
                ..Default::default()
            },
            y: CheckConfig {
                max_vl: 5,
                buffer: BufferConfig { max: 0.0, multiple: 0.0, decay: 0.0 },
                ..Default::default()
            },
        }
    }
}

/// Auto Clicker check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoClickerConfig {
    pub enabled: bool,
    pub a: CheckConfig, // Limit - CPS
    pub b: CheckConfig, // Deviation
    pub c: CheckConfig, // Rounded
    pub d: CheckConfig, // Skewness
    pub e: CheckConfig, // Variance
    pub f: CheckConfig, // Distinct
    pub g: CheckConfig, // Outliers
    pub h: CheckConfig, // Average Deviation
    pub i: CheckConfig, // Kurtosis
    pub j: CheckConfig, // Range
    pub k: CheckConfig, // Average Difference
    pub l: CheckConfig, // Kurtosis Difference
    pub m: CheckConfig, // Variance Difference
    pub n: CheckConfig, // Deviation Difference
    pub o: CheckConfig, // Spikes
    pub p: CheckConfig, // Identical
    pub q: CheckConfig, // Average Deviation
    pub r: CheckConfig, // Consistency
    pub s: CheckConfig, // Distinct
    pub t: CheckConfig, // Kurtosis
    /// Sample size for statistical analysis (default 20-50)
    pub sample_size: usize,
    /// CPS limit threshold
    pub cps_limit: f64,
    /// Standard deviation threshold
    pub std_dev_threshold: f64,
}

impl Default for AutoClickerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig { max_vl: 10, ..Default::default() },
            b: CheckConfig { max_vl: 10, ..Default::default() },
            c: CheckConfig { max_vl: 10, ..Default::default() },
            d: CheckConfig { max_vl: 10, ..Default::default() },
            e: CheckConfig { max_vl: 10, ..Default::default() },
            f: CheckConfig { max_vl: 10, ..Default::default() },
            g: CheckConfig { max_vl: 10, ..Default::default() },
            h: CheckConfig { max_vl: 10, ..Default::default() },
            i: CheckConfig { max_vl: 10, ..Default::default() },
            j: CheckConfig { max_vl: 10, ..Default::default() },
            k: CheckConfig { max_vl: 10, ..Default::default() },
            l: CheckConfig { max_vl: 10, ..Default::default() },
            m: CheckConfig { max_vl: 10, ..Default::default() },
            n: CheckConfig { max_vl: 10, ..Default::default() },
            o: CheckConfig { max_vl: 10, ..Default::default() },
            p: CheckConfig { max_vl: 10, ..Default::default() },
            q: CheckConfig { max_vl: 10, ..Default::default() },
            r: CheckConfig { max_vl: 10, ..Default::default() },
            s: CheckConfig { max_vl: 10, ..Default::default() },
            t: CheckConfig { max_vl: 10, ..Default::default() },
            sample_size: 20,
            cps_limit: 20.0,
            std_dev_threshold: 167.0,
        }
    }
}

/// Velocity check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityConfig {
    pub enabled: bool,
    pub a: CheckConfig, // Vertical
    pub b: CheckConfig, // Horizontal
    pub c: CheckConfig, // Ignored Vertical
    pub d: CheckConfig, // Horizontal (experimental)
    /// Minimum ratio for vertical velocity (0.999 = 99.9%)
    pub vertical_threshold: f64,
    /// Expected jump velocity threshold
    pub jump_threshold: f64,
}

impl Default for VelocityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig {
                max_vl: 5,
                buffer: BufferConfig { max: 2.0, multiple: 0.5, decay: 0.1 },
                ..Default::default()
            },
            b: CheckConfig { max_vl: 5, ..Default::default() },
            c: CheckConfig { max_vl: 3, ..Default::default() },
            d: CheckConfig { max_vl: 5, ..Default::default() },
            vertical_threshold: 0.999,
            jump_threshold: 0.419999,
        }
    }
}

/// Reach check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReachConfig {
    pub enabled: bool,
    pub a: CheckConfig, // History
    pub b: CheckConfig, // Simple
    /// Base reach distance (vanilla: 3.0)
    pub base_reach: f64,
    /// Lag compensation ticks
    pub lag_compensation_ticks: u32,
}

impl Default for ReachConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig { max_vl: 10, ..Default::default() },
            b: CheckConfig { max_vl: 10, ..Default::default() },
            base_reach: 3.0,
            lag_compensation_ticks: 20,
        }
    }
}

/// Hitbox check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitboxConfig {
    pub enabled: bool,
    pub a: CheckConfig, // History
    pub b: CheckConfig, // Simple
    /// Base angle threshold
    pub base_threshold: f64,
}

impl Default for HitboxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig { max_vl: 10, ..Default::default() },
            b: CheckConfig { max_vl: 10, ..Default::default() },
            base_threshold: 0.5,
        }
    }
}

/// Kill Aura check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillAuraConfig {
    pub enabled: bool,
    pub a: CheckConfig, // Post
    pub b: CheckConfig, // Acceleration
    pub c: CheckConfig, // Head Snap
    pub d: CheckConfig, // Multi Aura
    pub j: CheckConfig, // Frequency
    pub k: CheckConfig, // Pattern
    pub l: CheckConfig, // Strafe
}

impl Default for KillAuraConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig { max_vl: 5, ..Default::default() },
            b: CheckConfig { max_vl: 10, ..Default::default() },
            c: CheckConfig { max_vl: 10, ..Default::default() },
            d: CheckConfig { max_vl: 5, ..Default::default() },
            j: CheckConfig { max_vl: 10, ..Default::default() },
            k: CheckConfig { max_vl: 10, ..Default::default() },
            l: CheckConfig { max_vl: 10, ..Default::default() },
        }
    }
}

/// Criticals check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalsConfig {
    pub enabled: bool,
    pub a: CheckConfig, // Ground
    pub b: CheckConfig, // Modulo
}

impl Default for CriticalsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig { max_vl: 5, ..Default::default() },
            b: CheckConfig { max_vl: 5, ..Default::default() },
        }
    }
}

// ============================================================================
// Movement Check Configs
// ============================================================================

/// Flight check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightConfig {
    pub enabled: bool,
    pub a: CheckConfig, // Prediction [S]
    pub b: CheckConfig, // Prediction [C]
    pub c: CheckConfig, // Ascension
    pub d: CheckConfig, // Glide
    pub e: CheckConfig, // Hover
    pub f: CheckConfig, // Prediction (experimental)
    /// Gravity constant per tick
    pub gravity: f64,
    /// Drag per tick
    pub drag: f64,
}

impl Default for FlightConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig { max_vl: 5, ..Default::default() },
            b: CheckConfig { max_vl: 5, ..Default::default() },
            c: CheckConfig { max_vl: 5, ..Default::default() },
            d: CheckConfig { max_vl: 5, ..Default::default() },
            e: CheckConfig { max_vl: 5, ..Default::default() },
            f: CheckConfig { max_vl: 5, ..Default::default() },
            gravity: 0.08,
            drag: 0.98,
        }
    }
}

/// Speed check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedConfig {
    pub enabled: bool,
    pub a: CheckConfig, // Friction
    pub b: CheckConfig, // Ground
    pub c: CheckConfig, // Air
    pub d: CheckConfig, // Ground (alt)
    pub e: CheckConfig, // Prediction (experimental)
}

impl Default for SpeedConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig { max_vl: 10, ..Default::default() },
            b: CheckConfig { max_vl: 10, ..Default::default() },
            c: CheckConfig { max_vl: 10, ..Default::default() },
            d: CheckConfig { max_vl: 10, ..Default::default() },
            e: CheckConfig { max_vl: 10, ..Default::default() },
        }
    }
}

/// No Slow check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoSlowConfig {
    pub enabled: bool,
    pub a: CheckConfig, // Packet
    pub b: CheckConfig, // Soul Sand
    pub c: CheckConfig, // Web
}

impl Default for NoSlowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig { max_vl: 5, ..Default::default() },
            b: CheckConfig { max_vl: 10, ..Default::default() },
            c: CheckConfig { max_vl: 10, ..Default::default() },
        }
    }
}

/// Jump check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpConfig {
    pub enabled: bool,
    pub a: CheckConfig, // Motion
    pub b: CheckConfig, // Height
    pub c: CheckConfig, // Motion (alt)
    pub d: CheckConfig, // Motion (quaternary)
}

impl Default for JumpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig { max_vl: 5, ..Default::default() },
            b: CheckConfig { max_vl: 5, ..Default::default() },
            c: CheckConfig { max_vl: 5, ..Default::default() },
            d: CheckConfig { max_vl: 5, ..Default::default() },
        }
    }
}

/// Jesus (water walk) check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JesusConfig {
    pub enabled: bool,
    pub a: CheckConfig, // Ground
    pub b: CheckConfig, // Motion
    pub c: CheckConfig, // Y Motion
    pub d: CheckConfig, // Jump
    pub e: CheckConfig, // Speed
}

impl Default for JesusConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig { max_vl: 5, ..Default::default() },
            b: CheckConfig { max_vl: 5, ..Default::default() },
            c: CheckConfig { max_vl: 5, ..Default::default() },
            d: CheckConfig { max_vl: 5, ..Default::default() },
            e: CheckConfig { max_vl: 5, ..Default::default() },
        }
    }
}

/// Step check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepConfig {
    pub enabled: bool,
    pub a: CheckConfig, // Vanilla
    pub c: CheckConfig, // Motion
    /// Maximum vanilla step height
    pub max_step_height: f64,
}

impl Default for StepConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig { max_vl: 5, ..Default::default() },
            c: CheckConfig { max_vl: 5, ..Default::default() },
            max_step_height: 0.5,
        }
    }
}

/// Timer check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerConfig {
    pub enabled: bool,
    pub a: CheckConfig, // Average
    pub d: CheckConfig, // Balance
    /// Sample size for timer calculation
    pub sample_size: usize,
    /// Speed threshold for detection
    pub speed_threshold: f64,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig { max_vl: 10, ..Default::default() },
            d: CheckConfig { max_vl: 10, ..Default::default() },
            sample_size: 50,
            speed_threshold: 1.01, // 1% faster than normal
        }
    }
}

// ============================================================================
// Player Check Configs
// ============================================================================

/// Bad Packets check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadPacketsConfig {
    pub enabled: bool,
    // All Bad Packets checks use strict buffers
    pub checks: Vec<CheckConfig>,
}

impl Default for BadPacketsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            checks: vec![CheckConfig {
                buffer: BufferConfig { max: 1.0, multiple: 1.0, decay: 0.0 },
                ..Default::default()
            }; 30], // 30 different Bad Packets checks
        }
    }
}

/// Scaffold check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldConfig {
    pub enabled: bool,
    pub a: CheckConfig, // Interact
    pub b: CheckConfig, // Interact (non-exp)
    pub c: CheckConfig, // Sprint
    pub d: CheckConfig, // Rotations
    pub e: CheckConfig, // Rotations [2]
    pub f: CheckConfig, // Packet
    pub g: CheckConfig, // Speed
    pub h: CheckConfig, // Rotations [3]
    pub i: CheckConfig, // Acceleration
    pub j: CheckConfig, // Acceleration [2]
    pub k: CheckConfig, // Limit
    pub m: CheckConfig, // Expand
    pub n: CheckConfig, // Expand (experimental)
}

impl Default for ScaffoldConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig { max_vl: 5, ..Default::default() },
            b: CheckConfig { max_vl: 5, ..Default::default() },
            c: CheckConfig { max_vl: 5, ..Default::default() },
            d: CheckConfig { max_vl: 5, ..Default::default() },
            e: CheckConfig { max_vl: 5, ..Default::default() },
            f: CheckConfig { max_vl: 5, ..Default::default() },
            g: CheckConfig { max_vl: 5, ..Default::default() },
            h: CheckConfig { max_vl: 5, ..Default::default() },
            i: CheckConfig { max_vl: 5, ..Default::default() },
            j: CheckConfig { max_vl: 5, ..Default::default() },
            k: CheckConfig { max_vl: 5, ..Default::default() },
            m: CheckConfig { max_vl: 5, ..Default::default() },
            n: CheckConfig { max_vl: 5, ..Default::default() },
        }
    }
}

/// Ground Spoof check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundSpoofConfig {
    pub enabled: bool,
    pub a: CheckConfig, // Spoof
    pub b: CheckConfig, // Spoof (alt)
    pub c: CheckConfig, // Spoof (tertiary)
    /// Motion threshold for ground spoof detection
    pub motion_threshold: f64,
}

impl Default for GroundSpoofConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig { max_vl: 5, ..Default::default() },
            b: CheckConfig { max_vl: 5, ..Default::default() },
            c: CheckConfig { max_vl: 5, ..Default::default() },
            motion_threshold: 0.3116,
        }
    }
}

/// Fast Break check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastBreakConfig {
    pub enabled: bool,
    pub a: CheckConfig, // Delay
}

impl Default for FastBreakConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig { max_vl: 10, ..Default::default() },
        }
    }
}

/// Fast Place check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastPlaceConfig {
    pub enabled: bool,
    pub a: CheckConfig, // Delay
}

impl Default for FastPlaceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig { max_vl: 10, ..Default::default() },
        }
    }
}

/// Invalid check configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidConfig {
    pub enabled: bool,
    pub a: CheckConfig, // Invalid position
    pub b: CheckConfig, // Invalid position (alt)
    pub c: CheckConfig, // Y
    pub e: CheckConfig, // X/Z
    pub f: CheckConfig, // Spoofed Y
    pub i: CheckConfig, // Invalid Y
    pub j: CheckConfig, // Motion (experimental)
}

impl Default for InvalidConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            a: CheckConfig { max_vl: 5, ..Default::default() },
            b: CheckConfig { max_vl: 5, ..Default::default() },
            c: CheckConfig { max_vl: 5, ..Default::default() },
            e: CheckConfig { max_vl: 5, ..Default::default() },
            f: CheckConfig { max_vl: 5, ..Default::default() },
            i: CheckConfig { max_vl: 5, ..Default::default() },
            j: CheckConfig { max_vl: 5, ..Default::default() },
        }
    }
}

// ============================================================================
// Complete Configuration
// ============================================================================

/// Complete Vulcan configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulcanConfig {
    // Combat
    pub aim: AimConfig,
    pub autoclicker: AutoClickerConfig,
    pub velocity: VelocityConfig,
    pub reach: ReachConfig,
    pub hitbox: HitboxConfig,
    pub killaura: KillAuraConfig,
    pub criticals: CriticalsConfig,
    
    // Movement
    pub flight: FlightConfig,
    pub speed: SpeedConfig,
    pub noslow: NoSlowConfig,
    pub jump: JumpConfig,
    pub jesus: JesusConfig,
    pub step: StepConfig,
    pub timer: TimerConfig,
    
    // Player
    pub badpackets: BadPacketsConfig,
    pub scaffold: ScaffoldConfig,
    pub groundspoof: GroundSpoofConfig,
    pub fastbreak: FastBreakConfig,
    pub fastplace: FastPlaceConfig,
    pub invalid: InvalidConfig,
}

impl Default for VulcanConfig {
    fn default() -> Self {
        Self {
            aim: AimConfig::default(),
            autoclicker: AutoClickerConfig::default(),
            velocity: VelocityConfig::default(),
            reach: ReachConfig::default(),
            hitbox: HitboxConfig::default(),
            killaura: KillAuraConfig::default(),
            criticals: CriticalsConfig::default(),
            flight: FlightConfig::default(),
            speed: SpeedConfig::default(),
            noslow: NoSlowConfig::default(),
            jump: JumpConfig::default(),
            jesus: JesusConfig::default(),
            step: StepConfig::default(),
            timer: TimerConfig::default(),
            badpackets: BadPacketsConfig::default(),
            scaffold: ScaffoldConfig::default(),
            groundspoof: GroundSpoofConfig::default(),
            fastbreak: FastBreakConfig::default(),
            fastplace: FastPlaceConfig::default(),
            invalid: InvalidConfig::default(),
        }
    }
}

