//! Combat module configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatConfig {
    pub killaura: KillAuraConfig,
    pub aim: AimConfig,
    pub autoclicker: AutoClickerConfig,
    pub reach: ReachConfig,
    pub noswing: NoSwingConfig,
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            killaura: KillAuraConfig::default(),
            aim: AimConfig::default(),
            autoclicker: AutoClickerConfig::default(),
            reach: ReachConfig::default(),
            noswing: NoSwingConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillAuraConfig {
    pub enabled: bool,
    /// Minimum ms between attacks to different targets (multi-aura detection)
    pub multi_target_min_ms: i64,
    /// Post-attack threshold in ms
    pub post_threshold_ms: i64,
}

impl Default for KillAuraConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            multi_target_min_ms: 50,
            post_threshold_ms: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AimConfig {
    pub enabled: bool,
    /// Head snap threshold in degrees
    pub head_snap_threshold: f32,
    /// Minimum interval between valid head snaps (ms)
    pub head_snap_min_interval_ms: i64,
    /// Check for pitch spread anomalies
    pub check_pitch_spread: bool,
    /// Check for sensitivity/GCD anomalies
    pub check_sensitivity: bool,
    /// Check for modulo patterns (Vulcan-style)
    pub check_modulo: bool,
    /// Check for direction switching
    pub check_direction_switch: bool,
    /// Minimum combat ticks before checking aim
    pub min_combat_ticks: u32,
}

impl Default for AimConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            head_snap_threshold: 30.0,
            head_snap_min_interval_ms: 50,
            check_pitch_spread: true,
            check_sensitivity: true,
            check_modulo: true,
            check_direction_switch: true,
            min_combat_ticks: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoClickerConfig {
    pub enabled: bool,
    /// Maximum legitimate CPS
    pub max_cps: f64,
    /// Suspicious CPS threshold
    pub suspicious_cps: f64,
    /// Low standard deviation threshold (ms)
    pub low_std_dev_threshold: f64,
    /// Low variance threshold (ms²)
    pub low_variance_threshold: f64,
    /// Check for tick alignment
    pub check_tick_alignment: bool,
}

impl Default for AutoClickerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_cps: 20.0,
            suspicious_cps: 16.0,
            low_std_dev_threshold: 167.0,
            low_variance_threshold: 2000.0,
            check_tick_alignment: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReachConfig {
    pub enabled: bool,
    /// Maximum reach distance in blocks (vanilla 3.0)
    pub max_reach: f64,
    /// Critical reach threshold (definitely cheating)
    pub critical_reach: f64,
}

impl Default for ReachConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_reach: 3.5,
            critical_reach: 4.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoSwingConfig {
    pub enabled: bool,
    /// Threshold attacks without swing before flagging
    pub threshold: u32,
    /// Max time between swing and attack (ms)
    pub max_swing_age_ms: i64,
}

impl Default for NoSwingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 3,
            max_swing_age_ms: 500,
        }
    }
}
