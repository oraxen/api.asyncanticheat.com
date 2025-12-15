//! Movement module configuration

use serde::{Deserialize, Serialize};

/// Minecraft physics constants
pub const GRAVITY: f64 = 0.08;
pub const DRAG: f64 = 0.98;
pub const MAX_WALK_SPEED: f64 = 0.2873;
pub const MAX_SPRINT_SPEED: f64 = 0.3675;
pub const MAX_SNEAK_SPEED: f64 = 0.0663;
pub const STEP_HEIGHT_LIMIT: f64 = 0.6;
pub const GROUND_SPOOF_MOTION_THRESHOLD: f64 = 0.3116;
pub const EXPECTED_TPS: f64 = 20.0;
pub const EXPECTED_TICK_MS: f64 = 50.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementConfig {
    pub flight: FlightConfig,
    pub speed: SpeedConfig,
    pub nofall: NoFallConfig,
    pub timer: TimerConfig,
    pub step: StepConfig,
    pub groundspoof: GroundSpoofConfig,
    pub velocity: VelocityConfig,
    pub noslow: NoSlowConfig,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            flight: FlightConfig::default(),
            speed: SpeedConfig::default(),
            nofall: NoFallConfig::default(),
            timer: TimerConfig::default(),
            step: StepConfig::default(),
            groundspoof: GroundSpoofConfig::default(),
            velocity: VelocityConfig::default(),
            noslow: NoSlowConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightConfig {
    pub enabled: bool,
    /// Maximum Y velocity gain per tick without jumping (blocks/tick)
    pub max_y_gain: f64,
    /// Maximum ticks of sustained upward movement
    pub max_ascend_ticks: u32,
    /// Maximum hover ticks (near-zero Y velocity)
    pub max_hover_ticks: u32,
    /// Hover Y threshold (considered hovering if Y delta below this)
    pub hover_threshold: f64,
    /// Y prediction tolerance (deviation from expected gravity)
    pub y_prediction_tolerance: f64,
}

impl Default for FlightConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_y_gain: 0.42, // Jump velocity
            max_ascend_ticks: 8,
            max_hover_ticks: 6,
            hover_threshold: 0.005,
            y_prediction_tolerance: 0.03,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedConfig {
    pub enabled: bool,
    /// Maximum walk speed (blocks/tick)
    pub max_walk_speed: f64,
    /// Maximum sprint speed (blocks/tick)
    pub max_sprint_speed: f64,
    /// Maximum sneak speed (blocks/tick)
    pub max_sneak_speed: f64,
    /// Speed tolerance factor (1.0 = exact, 1.1 = 10% tolerance)
    pub tolerance: f64,
}

impl Default for SpeedConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_walk_speed: MAX_WALK_SPEED,
            max_sprint_speed: MAX_SPRINT_SPEED,
            max_sneak_speed: MAX_SNEAK_SPEED,
            tolerance: 1.05,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoFallConfig {
    pub enabled: bool,
    /// Minimum fall distance before checking ground claims (blocks)
    pub min_fall_distance: f64,
    /// Maximum Y velocity for valid ground claim
    pub ground_claim_max_velocity: f64,
    /// Consecutive invalid ground claims before flagging
    pub consecutive_threshold: u32,
}

impl Default for NoFallConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_fall_distance: 3.0,
            ground_claim_max_velocity: -0.0784,
            consecutive_threshold: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerConfig {
    pub enabled: bool,
    /// Expected milliseconds per tick
    pub expected_tick_ms: f64,
    /// Maximum packet rate deviation (percentage)
    pub max_deviation_percent: f64,
    /// Minimum samples before checking
    pub min_samples: usize,
    /// Window size for rate calculation (ms)
    pub window_ms: i64,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            expected_tick_ms: EXPECTED_TICK_MS,
            max_deviation_percent: 15.0,
            min_samples: 20,
            window_ms: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepConfig {
    pub enabled: bool,
    /// Maximum step height (blocks)
    pub max_step_height: f64,
    /// Require player to be on ground before step
    pub require_ground: bool,
}

impl Default for StepConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_step_height: STEP_HEIGHT_LIMIT,
            require_ground: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundSpoofConfig {
    pub enabled: bool,
    /// Minimum downward velocity to be considered falling
    pub fall_threshold: f64,
    /// Maximum downward velocity where ground claim is valid
    pub ground_claim_threshold: f64,
    /// Consecutive spoofs before flagging
    pub consecutive_threshold: u32,
}

impl Default for GroundSpoofConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fall_threshold: GROUND_SPOOF_MOTION_THRESHOLD,
            ground_claim_threshold: -0.0784,
            consecutive_threshold: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityConfig {
    pub enabled: bool,
    /// Minimum velocity magnitude to track
    pub min_velocity: f64,
    /// Maximum ticks to respond to velocity
    pub max_response_ticks: u32,
    /// Minimum percentage of velocity that must be applied
    pub min_velocity_percent: f64,
    /// Number of ignored velocities before flagging
    pub ignore_threshold: u32,
}

impl Default for VelocityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_velocity: 0.1,
            max_response_ticks: 20,
            min_velocity_percent: 50.0,
            ignore_threshold: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoSlowConfig {
    pub enabled: bool,
    /// Speed multiplier when using items (vanilla is 0.2)
    pub using_item_multiplier: f64,
    /// Speed multiplier when sneaking (vanilla is ~0.3)
    pub sneak_multiplier: f64,
    /// Tolerance factor
    pub tolerance: f64,
}

impl Default for NoSlowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            using_item_multiplier: 0.2,
            sneak_multiplier: 0.3,
            tolerance: 1.1,
        }
    }
}
