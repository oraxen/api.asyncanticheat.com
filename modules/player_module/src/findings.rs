//! Player module findings and feature IDs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Feature identifiers for player checks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeatureId {
    // BadPackets
    BadPacketsPitch,
    BadPacketsNaN,
    BadPacketsAbilities,
    BadPacketsInstantBreak,
    BadPacketsSlot,
    BadPacketsFlyingFlood,

    // Scaffold
    ScaffoldAirborne,
    ScaffoldSprint,

    // FastPlace
    FastPlace,
    FastPlaceCritical,

    // FastBreak
    FastBreak,
    FastBreakCritical,

    // Interact
    InteractAngle,
    InteractImpossible,

    // Inventory
    InventoryFastClick,
}

impl FeatureId {
    pub fn name(&self) -> &'static str {
        match self {
            Self::BadPacketsPitch => "badpackets_pitch",
            Self::BadPacketsNaN => "badpackets_nan",
            Self::BadPacketsAbilities => "badpackets_abilities",
            Self::BadPacketsInstantBreak => "badpackets_instant_break",
            Self::BadPacketsSlot => "badpackets_slot",
            Self::BadPacketsFlyingFlood => "badpackets_flying_flood",
            Self::ScaffoldAirborne => "scaffold_airborne",
            Self::ScaffoldSprint => "scaffold_sprint",
            Self::FastPlace => "fastplace",
            Self::FastPlaceCritical => "fastplace_critical",
            Self::FastBreak => "fastbreak",
            Self::FastBreakCritical => "fastbreak_critical",
            Self::InteractAngle => "interact_angle",
            Self::InteractImpossible => "interact_impossible",
            Self::InventoryFastClick => "inventory_fastclick",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            Self::BadPacketsPitch | Self::BadPacketsNaN | Self::BadPacketsAbilities |
            Self::BadPacketsInstantBreak | Self::BadPacketsSlot | Self::BadPacketsFlyingFlood => "badpackets",
            Self::ScaffoldAirborne | Self::ScaffoldSprint => "scaffold",
            Self::FastPlace | Self::FastPlaceCritical => "fastplace",
            Self::FastBreak | Self::FastBreakCritical => "fastbreak",
            Self::InteractAngle | Self::InteractImpossible => "interact",
            Self::InventoryFastClick => "inventory",
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
