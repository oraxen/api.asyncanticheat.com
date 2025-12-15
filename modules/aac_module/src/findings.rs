//! Findings and Feature IDs from AAC5
//!
//! AAC reports detections as eA enum values (me.konsolas.aac.eA)

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// AAC Feature IDs (from me.konsolas.aac.eA)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FeatureId {
    // Delays
    AacDelaysBreak,
    AacDelaysPlace,
    AacDelaysConsume,
    AacDelaysBow,
    AacDelaysRegen,
    AacDelaysUnsneak,
    AacDelaysRelease,
    AacDelaysBreakdelay,

    // Move
    AacMoveNofall,
    AacMoveVel,
    AacMoveVehicle,
    AacMoveElytra,
    AacMoveWater,
    AacMoveLava,
    AacMoveNoslow,
    AacMoveGeneric,
    AacMoveTimer,
    AacMoveInf,

    // Aimbot
    AacAimbotPitchSpread,
    AacAimbotSensMismatch,
    AacAimbotSensXOor,
    AacAimbotSensYOor,
    AacAimbotBadDeltaX,
    AacAimbotBadDeltaY,
    AacAimbotHeadSnap,
    AacAimbotZeroPoint,

    // Autoclicker
    AacClickCps,
    AacClickVar,
    AacClickTickDelay,
    AacClickNoswing,
    AacClickTiming,

    // Hitbox
    AacHitboxCount,
    AacHitboxMiss,
    AacHitboxWalls,
    AacHitboxReach,
    AacHitboxInvalid,

    // Interact
    AacInteractGen,
    AacInteractBreak,
    AacInteractPlace,

    // Misc
    AacMiscAbilities,
    AacMiscPitch,
    AacMiscRotation,
}

impl FeatureId {
    /// Get the detector name string
    pub fn detector_name(&self) -> &'static str {
        match self {
            // Delays
            Self::AacDelaysBreak => "aac_delays_break",
            Self::AacDelaysPlace => "aac_delays_place",
            Self::AacDelaysConsume => "aac_delays_consume",
            Self::AacDelaysBow => "aac_delays_bow",
            Self::AacDelaysRegen => "aac_delays_regen",
            Self::AacDelaysUnsneak => "aac_delays_unsneak",
            Self::AacDelaysRelease => "aac_delays_release",
            Self::AacDelaysBreakdelay => "aac_delays_breakdelay",

            // Move
            Self::AacMoveNofall => "aac_move_nofall",
            Self::AacMoveVel => "aac_move_velocity",
            Self::AacMoveVehicle => "aac_move_vehicle",
            Self::AacMoveElytra => "aac_move_elytra",
            Self::AacMoveWater => "aac_move_water",
            Self::AacMoveLava => "aac_move_lava",
            Self::AacMoveNoslow => "aac_move_noslow",
            Self::AacMoveGeneric => "aac_move_generic",
            Self::AacMoveTimer => "aac_move_timer",
            Self::AacMoveInf => "aac_move_infinite",

            // Aimbot
            Self::AacAimbotPitchSpread => "aac_aimbot_pitch_spread",
            Self::AacAimbotSensMismatch => "aac_aimbot_sens_mismatch",
            Self::AacAimbotSensXOor => "aac_aimbot_sens_x_oor",
            Self::AacAimbotSensYOor => "aac_aimbot_sens_y_oor",
            Self::AacAimbotBadDeltaX => "aac_aimbot_bad_delta_x",
            Self::AacAimbotBadDeltaY => "aac_aimbot_bad_delta_y",
            Self::AacAimbotHeadSnap => "aac_aimbot_head_snap",
            Self::AacAimbotZeroPoint => "aac_aimbot_zero_point",

            // Autoclicker
            Self::AacClickCps => "aac_click_cps",
            Self::AacClickVar => "aac_click_variance",
            Self::AacClickTickDelay => "aac_click_tick_delay",
            Self::AacClickNoswing => "aac_click_noswing",
            Self::AacClickTiming => "aac_click_timing",

            // Hitbox
            Self::AacHitboxCount => "aac_hitbox_count",
            Self::AacHitboxMiss => "aac_hitbox_miss",
            Self::AacHitboxWalls => "aac_hitbox_walls",
            Self::AacHitboxReach => "aac_hitbox_reach",
            Self::AacHitboxInvalid => "aac_hitbox_invalid",

            // Interact
            Self::AacInteractGen => "aac_interact_generic",
            Self::AacInteractBreak => "aac_interact_break",
            Self::AacInteractPlace => "aac_interact_place",

            // Misc
            Self::AacMiscAbilities => "aac_misc_abilities",
            Self::AacMiscPitch => "aac_misc_pitch",
            Self::AacMiscRotation => "aac_misc_rotation",
        }
    }

    /// Get the check category
    pub fn category(&self) -> &'static str {
        match self {
            Self::AacDelaysBreak
            | Self::AacDelaysPlace
            | Self::AacDelaysConsume
            | Self::AacDelaysBow
            | Self::AacDelaysRegen
            | Self::AacDelaysUnsneak
            | Self::AacDelaysRelease
            | Self::AacDelaysBreakdelay => "delays",

            Self::AacMoveNofall
            | Self::AacMoveVel
            | Self::AacMoveVehicle
            | Self::AacMoveElytra
            | Self::AacMoveWater
            | Self::AacMoveLava
            | Self::AacMoveNoslow
            | Self::AacMoveGeneric
            | Self::AacMoveTimer
            | Self::AacMoveInf => "move",

            Self::AacAimbotPitchSpread
            | Self::AacAimbotSensMismatch
            | Self::AacAimbotSensXOor
            | Self::AacAimbotSensYOor
            | Self::AacAimbotBadDeltaX
            | Self::AacAimbotBadDeltaY
            | Self::AacAimbotHeadSnap
            | Self::AacAimbotZeroPoint => "aimbot",

            Self::AacClickCps
            | Self::AacClickVar
            | Self::AacClickTickDelay
            | Self::AacClickNoswing
            | Self::AacClickTiming => "autoclicker",

            Self::AacHitboxCount
            | Self::AacHitboxMiss
            | Self::AacHitboxWalls
            | Self::AacHitboxReach
            | Self::AacHitboxInvalid => "hitbox",

            Self::AacInteractGen
            | Self::AacInteractBreak
            | Self::AacInteractPlace => "interact",

            Self::AacMiscAbilities | Self::AacMiscPitch | Self::AacMiscRotation => "misc",
        }
    }
}

/// Finding severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

/// A finding/detection from AAC checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Player UUID
    pub player_uuid: Uuid,
    /// Feature ID that triggered
    pub feature: FeatureId,
    /// Severity level
    pub severity: Severity,
    /// Human-readable title
    pub title: String,
    /// Detailed description
    pub description: Option<String>,
    /// Signal strength / value that triggered
    pub value: f32,
    /// Current VL after this finding
    pub vl: f32,
    /// Whether mitigation was triggered
    pub mitigated: bool,
    /// Timestamp
    pub timestamp_ms: i64,
    /// Additional evidence
    pub evidence: Option<serde_json::Value>,
}

impl Finding {
    pub fn new(
        player_uuid: Uuid,
        feature: FeatureId,
        value: f32,
        vl: f32,
        mitigated: bool,
        timestamp_ms: i64,
    ) -> Self {
        let severity = Self::calculate_severity(vl, mitigated);
        let title = Self::generate_title(&feature, value);

        Self {
            player_uuid,
            feature,
            severity,
            title,
            description: None,
            value,
            vl,
            mitigated,
            timestamp_ms,
            evidence: None,
        }
    }

    fn calculate_severity(vl: f32, mitigated: bool) -> Severity {
        if mitigated {
            if vl >= 0.9 {
                Severity::Critical
            } else {
                Severity::High
            }
        } else if vl >= 0.7 {
            Severity::Medium
        } else if vl >= 0.3 {
            Severity::Low
        } else {
            Severity::Info
        }
    }

    fn generate_title(feature: &FeatureId, value: f32) -> String {
        match feature {
            FeatureId::AacDelaysBreak => format!("Fast break: {:.1}x speed", value),
            FeatureId::AacDelaysPlace => format!("Fast place: {:.1}x speed", value),
            FeatureId::AacDelaysConsume => format!("Fast consume: {:.1}x speed", value),
            FeatureId::AacDelaysBow => format!("Fast bow: {:.1}x speed", value),
            FeatureId::AacDelaysRegen => format!("Fast regen: {:.1}x speed", value),
            FeatureId::AacDelaysUnsneak => format!("Fast unsneak: {:.1}x speed", value),
            FeatureId::AacDelaysRelease => format!("Fast release: {:.1}x speed", value),
            FeatureId::AacDelaysBreakdelay => format!("Break delay: {:.1}x speed", value),

            FeatureId::AacMoveNofall => "NoFall detected".to_string(),
            FeatureId::AacMoveVel => format!("Velocity ignored: {:.1}%", value * 100.0),
            FeatureId::AacMoveVehicle => format!("Invalid vehicle move: {:.2}", value),
            FeatureId::AacMoveElytra => format!("Invalid elytra: {:.2} b/s", value),
            FeatureId::AacMoveWater => format!("Invalid water move: {:.2}", value),
            FeatureId::AacMoveLava => format!("Invalid lava move: {:.2}", value),
            FeatureId::AacMoveNoslow => format!("NoSlow: {:.2} b/s", value),
            FeatureId::AacMoveGeneric => format!("Invalid move: {:.2}", value),
            FeatureId::AacMoveTimer => format!("Timer: {:.1}% faster", value * 100.0),
            FeatureId::AacMoveInf => format!("Infinite move: {:.2}", value),

            FeatureId::AacAimbotPitchSpread => format!("Pitch spread: {:.2}°", value),
            FeatureId::AacAimbotSensMismatch => format!("Sens mismatch: {:.2}", value),
            FeatureId::AacAimbotSensXOor => format!("X sensitivity OOR: {:.2}", value),
            FeatureId::AacAimbotSensYOor => format!("Y sensitivity OOR: {:.2}", value),
            FeatureId::AacAimbotBadDeltaX => format!("Bad delta X: {:.2}°", value),
            FeatureId::AacAimbotBadDeltaY => format!("Bad delta Y: {:.2}°", value),
            FeatureId::AacAimbotHeadSnap => format!("Head snap: {:.1}°", value),
            FeatureId::AacAimbotZeroPoint => "Zero-point anchor".to_string(),

            FeatureId::AacClickCps => format!("{:.1} CPS", value),
            FeatureId::AacClickVar => format!("Low variance: {:.3}", value),
            FeatureId::AacClickTickDelay => format!("Tick-aligned: {:.2}", value),
            FeatureId::AacClickNoswing => "Attack without swing".to_string(),
            FeatureId::AacClickTiming => format!("Suspicious timing: {:.2}", value),

            FeatureId::AacHitboxCount => format!("Hit count: {}", value as i32),
            FeatureId::AacHitboxMiss => format!("Miss ratio: {:.1}%", value * 100.0),
            FeatureId::AacHitboxWalls => "Hit through walls".to_string(),
            FeatureId::AacHitboxReach => format!("Reach: {:.2} blocks", value),
            FeatureId::AacHitboxInvalid => format!("Invalid hitbox offset: {:.2}", value),

            FeatureId::AacInteractGen => format!("Invalid interact: {:.2}°", value),
            FeatureId::AacInteractBreak => format!("Invalid break angle: {:.2}°", value),
            FeatureId::AacInteractPlace => format!("Invalid place angle: {:.2}°", value),

            FeatureId::AacMiscAbilities => "Invalid abilities".to_string(),
            FeatureId::AacMiscPitch => format!("Invalid pitch: {:.1}°", value),
            FeatureId::AacMiscRotation => format!("Rotation rate: {:.1}°/s", value),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_evidence(mut self, evidence: serde_json::Value) -> Self {
        self.evidence = Some(evidence);
        self
    }
}

