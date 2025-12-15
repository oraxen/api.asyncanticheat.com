//! Movement module findings and feature IDs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Feature identifiers for movement checks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeatureId {
    // Flight
    FlightYPrediction,
    FlightSustainedAscend,
    FlightHover,

    // Speed
    SpeedHorizontal,
    SpeedSprint,
    SpeedSneak,

    // NoFall
    NoFallInvalidGround,
    NoFallFakeDamage,

    // Timer
    TimerFast,
    TimerSlow,

    // Step
    StepHeight,
    StepNoGround,

    // GroundSpoof
    GroundSpoofFalling,
    GroundSpoofAscending,

    // Velocity
    VelocityIgnored,
    VelocityPartial,

    // NoSlow
    NoSlowUsingItem,
    NoSlowSneaking,
}

impl FeatureId {
    pub fn name(&self) -> &'static str {
        match self {
            Self::FlightYPrediction => "flight_yprediction",
            Self::FlightSustainedAscend => "flight_ascend",
            Self::FlightHover => "flight_hover",
            Self::SpeedHorizontal => "speed_horizontal",
            Self::SpeedSprint => "speed_sprint",
            Self::SpeedSneak => "speed_sneak",
            Self::NoFallInvalidGround => "nofall_ground",
            Self::NoFallFakeDamage => "nofall_damage",
            Self::TimerFast => "timer_fast",
            Self::TimerSlow => "timer_slow",
            Self::StepHeight => "step_height",
            Self::StepNoGround => "step_noground",
            Self::GroundSpoofFalling => "groundspoof_falling",
            Self::GroundSpoofAscending => "groundspoof_ascending",
            Self::VelocityIgnored => "velocity_ignored",
            Self::VelocityPartial => "velocity_partial",
            Self::NoSlowUsingItem => "noslow_item",
            Self::NoSlowSneaking => "noslow_sneak",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            Self::FlightYPrediction | Self::FlightSustainedAscend | Self::FlightHover => "flight",
            Self::SpeedHorizontal | Self::SpeedSprint | Self::SpeedSneak => "speed",
            Self::NoFallInvalidGround | Self::NoFallFakeDamage => "nofall",
            Self::TimerFast | Self::TimerSlow => "timer",
            Self::StepHeight | Self::StepNoGround => "step",
            Self::GroundSpoofFalling | Self::GroundSpoofAscending => "groundspoof",
            Self::VelocityIgnored | Self::VelocityPartial => "velocity",
            Self::NoSlowUsingItem | Self::NoSlowSneaking => "noslow",
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
