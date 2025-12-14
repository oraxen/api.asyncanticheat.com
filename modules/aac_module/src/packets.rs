//! Packet structures for AAC module
//!
//! These represent the packet types AAC5 processes:
//! - POSITION, LOOK, POSITION_LOOK (movement)
//! - USE_ENTITY (attacks)
//! - ARM_ANIMATION (swings)
//! - ENTITY_VELOCITY (knockback)
//! - Block interactions

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Packet record from the batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketRecord {
    /// Timestamp in milliseconds
    pub timestamp_ms: i64,
    /// Player UUID
    pub player_uuid: Uuid,
    /// Player name
    pub player_name: Option<String>,
    /// Packet type
    pub packet_type: String,
    /// Packet data (JSON)
    pub data: serde_json::Value,
}

/// Position packet data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub on_ground: bool,
}

/// Look packet data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookPacket {
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

/// Position+Look combined packet data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionLookPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

/// Use Entity (attack) packet data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseEntityPacket {
    pub entity_id: i32,
    pub action: String, // ATTACK, INTERACT, INTERACT_AT
    pub target_x: Option<f64>,
    pub target_y: Option<f64>,
    pub target_z: Option<f64>,
    pub hand: Option<String>,
}

/// Arm Animation (swing) packet data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmAnimationPacket {
    pub hand: String, // MAIN_HAND, OFF_HAND
}

/// Entity Velocity packet data (from server)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityVelocityPacket {
    pub entity_id: i32,
    pub velocity_x: f64,
    pub velocity_y: f64,
    pub velocity_z: f64,
}

/// Block Dig packet data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDigPacket {
    pub status: String, // START_DESTROY, ABORT_DESTROY, STOP_DESTROY
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub face: String,
}

/// Block Place packet data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPlacePacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub face: String,
    pub hand: String,
}

/// Item Use packet data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemUsePacket {
    pub hand: String,
    pub item_type: Option<String>,
}

/// Sneak packet data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SneakPacket {
    pub sneaking: bool,
}

/// Player Abilities packet data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilitiesPacket {
    pub flying: bool,
    pub allow_flying: bool,
    pub invulnerable: bool,
    pub instant_break: bool,
}

/// Parsed packet with typed data
#[derive(Debug, Clone)]
pub enum ParsedPacket {
    Position(PositionPacket),
    Look(LookPacket),
    PositionLook(PositionLookPacket),
    UseEntity(UseEntityPacket),
    ArmAnimation(ArmAnimationPacket),
    EntityVelocity(EntityVelocityPacket),
    BlockDig(BlockDigPacket),
    BlockPlace(BlockPlacePacket),
    ItemUse(ItemUsePacket),
    Sneak(SneakPacket),
    Abilities(AbilitiesPacket),
    Unknown(String),
}

impl PacketRecord {
    /// Parse the packet data into a typed variant
    pub fn parse(&self) -> ParsedPacket {
        match self.packet_type.as_str() {
            "POSITION" | "FLYING_POSITION" => {
                if let Ok(p) = serde_json::from_value(self.data.clone()) {
                    ParsedPacket::Position(p)
                } else {
                    ParsedPacket::Unknown(self.packet_type.clone())
                }
            }
            "LOOK" | "FLYING_LOOK" => {
                if let Ok(p) = serde_json::from_value(self.data.clone()) {
                    ParsedPacket::Look(p)
                } else {
                    ParsedPacket::Unknown(self.packet_type.clone())
                }
            }
            "POSITION_LOOK" | "FLYING_POSITION_LOOK" => {
                if let Ok(p) = serde_json::from_value(self.data.clone()) {
                    ParsedPacket::PositionLook(p)
                } else {
                    ParsedPacket::Unknown(self.packet_type.clone())
                }
            }
            "USE_ENTITY" => {
                if let Ok(p) = serde_json::from_value(self.data.clone()) {
                    ParsedPacket::UseEntity(p)
                } else {
                    ParsedPacket::Unknown(self.packet_type.clone())
                }
            }
            "ARM_ANIMATION" | "ANIMATION" => {
                if let Ok(p) = serde_json::from_value(self.data.clone()) {
                    ParsedPacket::ArmAnimation(p)
                } else {
                    ParsedPacket::Unknown(self.packet_type.clone())
                }
            }
            "ENTITY_VELOCITY" => {
                if let Ok(p) = serde_json::from_value(self.data.clone()) {
                    ParsedPacket::EntityVelocity(p)
                } else {
                    ParsedPacket::Unknown(self.packet_type.clone())
                }
            }
            "BLOCK_DIG" => {
                if let Ok(p) = serde_json::from_value(self.data.clone()) {
                    ParsedPacket::BlockDig(p)
                } else {
                    ParsedPacket::Unknown(self.packet_type.clone())
                }
            }
            "BLOCK_PLACE" => {
                if let Ok(p) = serde_json::from_value(self.data.clone()) {
                    ParsedPacket::BlockPlace(p)
                } else {
                    ParsedPacket::Unknown(self.packet_type.clone())
                }
            }
            "USE_ITEM" => {
                // USE_ITEM is for using items (eating, drinking, etc.), not block placement
                if let Ok(p) = serde_json::from_value(self.data.clone()) {
                    ParsedPacket::ItemUse(p)
                } else {
                    ParsedPacket::Unknown(self.packet_type.clone())
                }
            }
            "HELD_ITEM_SLOT" | "BLOCK_ITEM_SWITCH" => {
                // These are hotbar slot changes, not item usage - treat as unknown for now
                // since they don't map to ItemUse semantics
                ParsedPacket::Unknown(self.packet_type.clone())
            }
            "ENTITY_ACTION" => {
                // Check for sneak action
                if let Some(action) = self.data.get("action").and_then(|v| v.as_str()) {
                    if action.contains("SNEAK") {
                        let sneaking = action.contains("START");
                        return ParsedPacket::Sneak(SneakPacket { sneaking });
                    }
                }
                ParsedPacket::Unknown(self.packet_type.clone())
            }
            "ABILITIES" => {
                if let Ok(p) = serde_json::from_value(self.data.clone()) {
                    ParsedPacket::Abilities(p)
                } else {
                    ParsedPacket::Unknown(self.packet_type.clone())
                }
            }
            _ => ParsedPacket::Unknown(self.packet_type.clone()),
        }
    }
}

/// Location with rotation
#[derive(Debug, Clone, Copy, Default)]
pub struct Location {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

impl Location {
    pub fn distance_squared(&self, other: &Location) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    pub fn distance(&self, other: &Location) -> f64 {
        self.distance_squared(other).sqrt()
    }

    pub fn horizontal_distance_squared(&self, other: &Location) -> f64 {
        let dx = self.x - other.x;
        let dz = self.z - other.z;
        dx * dx + dz * dz
    }

    pub fn horizontal_distance(&self, other: &Location) -> f64 {
        self.horizontal_distance_squared(other).sqrt()
    }
}

