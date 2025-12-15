//! Combat module findings and feature IDs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Feature identifiers for combat checks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeatureId {
    // KillAura
    KillAuraMultiTarget,
    KillAuraPost,

    // Aim
    AimHeadSnap,
    AimPitchSpread,
    AimSensitivity,
    AimModulo,
    AimDirectionSwitch,
    AimRepeatedYaw,

    // AutoClicker
    AutoClickerCps,
    AutoClickerTiming,
    AutoClickerVariance,
    AutoClickerKurtosis,
    AutoClickerTickAlign,

    // Reach
    ReachDistance,
    ReachCritical,

    // NoSwing
    NoSwing,
}

impl FeatureId {
    pub fn name(&self) -> &'static str {
        match self {
            Self::KillAuraMultiTarget => "killaura_multi",
            Self::KillAuraPost => "killaura_post",
            Self::AimHeadSnap => "aim_headsnap",
            Self::AimPitchSpread => "aim_pitchspread",
            Self::AimSensitivity => "aim_sensitivity",
            Self::AimModulo => "aim_modulo",
            Self::AimDirectionSwitch => "aim_dirswitch",
            Self::AimRepeatedYaw => "aim_repeated_yaw",
            Self::AutoClickerCps => "autoclicker_cps",
            Self::AutoClickerTiming => "autoclicker_timing",
            Self::AutoClickerVariance => "autoclicker_variance",
            Self::AutoClickerKurtosis => "autoclicker_kurtosis",
            Self::AutoClickerTickAlign => "autoclicker_tickalign",
            Self::ReachDistance => "reach_distance",
            Self::ReachCritical => "reach_critical",
            Self::NoSwing => "noswing",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            Self::KillAuraMultiTarget | Self::KillAuraPost => "killaura",
            Self::AimHeadSnap | Self::AimPitchSpread | Self::AimSensitivity |
            Self::AimModulo | Self::AimDirectionSwitch | Self::AimRepeatedYaw => "aim",
            Self::AutoClickerCps | Self::AutoClickerTiming | Self::AutoClickerVariance |
            Self::AutoClickerKurtosis | Self::AutoClickerTickAlign => "autoclicker",
            Self::ReachDistance | Self::ReachCritical => "reach",
            Self::NoSwing => "noswing",
        }
    }
}

/// A detection finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub player_uuid: Uuid,
    pub feature_id: FeatureId,
    pub value: f64,
    pub vl: u32,
    pub max_vl: u32,
    pub should_mitigate: bool,
    pub timestamp_ms: i64,
    pub description: Option<String>,
    pub evidence: Option<serde_json::Value>,
}

impl Finding {
    pub fn new(
        player_uuid: Uuid,
        feature_id: FeatureId,
        value: f64,
        vl: u32,
        max_vl: u32,
        timestamp_ms: i64,
    ) -> Self {
        Self {
            player_uuid,
            feature_id,
            value,
            vl,
            max_vl,
            should_mitigate: vl >= max_vl,
            timestamp_ms,
            description: None,
            evidence: None,
        }
    }

    pub fn with_description(mut self, desc: String) -> Self {
        self.description = Some(desc);
        self
    }

    pub fn with_evidence(mut self, evidence: serde_json::Value) -> Self {
        self.evidence = Some(evidence);
        self
    }

    pub fn with_mitigate(mut self, should_mitigate: bool) -> Self {
        self.should_mitigate = should_mitigate;
        self
    }
}
