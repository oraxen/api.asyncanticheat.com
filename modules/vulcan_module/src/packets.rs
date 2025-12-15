//! Packet structures for Vulcan module
//!
//! Vulcan uses PacketEvents library for packet interception

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Packet record from the batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketRecord {
    pub timestamp_ms: i64,
    pub player_uuid: Uuid,
    pub player_name: Option<String>,
    pub packet_type: String,
    pub data: serde_json::Value,
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
    pub fn horizontal_distance(&self, other: &Location) -> f64 {
        let dx = self.x - other.x;
        let dz = self.z - other.z;
        (dx * dx + dz * dz).sqrt()
    }

    pub fn distance_3d(&self, other: &Location) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Position packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub on_ground: bool,
}

/// Look packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookPacket {
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

/// Position + Look packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionLookPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

/// Use Entity (attack/interact) packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseEntityPacket {
    pub entity_id: i32,
    pub action: String, // ATTACK, INTERACT, INTERACT_AT
    pub target_x: Option<f64>,
    pub target_y: Option<f64>,
    pub target_z: Option<f64>,
    pub hand: Option<String>,
}

/// Arm Animation (swing) packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmAnimationPacket {
    pub hand: String,
}

/// Entity Velocity packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityVelocityPacket {
    pub entity_id: i32,
    pub velocity_x: f64,
    pub velocity_y: f64,
    pub velocity_z: f64,
}

/// Block Dig packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDigPacket {
    pub status: String, // START_DIGGING, STOP_DIGGING, etc.
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub face: String,
}

/// Block Place packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPlacePacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub face: String,
    pub hand: String,
}

/// Entity Action packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityActionPacket {
    pub action: String, // START_SNEAKING, STOP_SNEAKING, START_SPRINTING, etc.
    pub jump_boost: Option<i32>,
}

/// Held Item Slot packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeldItemSlotPacket {
    pub slot: i32,
}

/// Player Abilities packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilitiesPacket {
    pub flying: bool,
    pub allow_flying: bool,
    pub invulnerable: bool,
    pub instant_break: bool,
}

/// Steer Vehicle packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteerVehiclePacket {
    pub sideways: f32,
    pub forward: f32,
    pub jumping: bool,
    pub dismounting: bool,
}

/// Keep Alive packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeepAlivePacket {
    pub id: i64,
}

/// Window Click packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowClickPacket {
    pub window_id: i32,
    pub slot: i32,
    pub button: i32,
    pub mode: i32,
}

/// Flying packet (ground only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlyingPacket {
    pub on_ground: bool,
}

/// Parsed packet with typed data
#[derive(Debug, Clone)]
pub enum ParsedPacket {
    Position(PositionPacket),
    Look(LookPacket),
    PositionLook(PositionLookPacket),
    Flying(FlyingPacket),
    UseEntity(UseEntityPacket),
    ArmAnimation(ArmAnimationPacket),
    EntityVelocity(EntityVelocityPacket),
    BlockDig(BlockDigPacket),
    BlockPlace(BlockPlacePacket),
    EntityAction(EntityActionPacket),
    HeldItemSlot(HeldItemSlotPacket),
    Abilities(AbilitiesPacket),
    SteerVehicle(SteerVehiclePacket),
    KeepAlive(KeepAlivePacket),
    WindowClick(WindowClickPacket),
    Unknown(String),
}

impl PacketRecord {
    pub fn parse(&self) -> ParsedPacket {
        match self.packet_type.as_str() {
            // PacketEvents names (modern)
            "PLAYER_POSITION" | "PLAYER_FLYING_POSITION"
            // Legacy/normalized names
            | "POSITION" | "FLYING_POSITION" => {
                serde_json::from_value(self.data.clone())
                    .map(ParsedPacket::Position)
                    .unwrap_or(ParsedPacket::Unknown(self.packet_type.clone()))
            }
            // PacketEvents names (modern)
            "PLAYER_ROTATION" | "PLAYER_FLYING_LOOK"
            // Legacy/normalized names
            | "LOOK" | "FLYING_LOOK" => {
                serde_json::from_value(self.data.clone())
                    .map(ParsedPacket::Look)
                    .unwrap_or(ParsedPacket::Unknown(self.packet_type.clone()))
            }
            // PacketEvents names (modern)
            "PLAYER_POSITION_AND_ROTATION" | "PLAYER_POSITION_AND_LOOK" | "PLAYER_FLYING_POSITION_AND_LOOK"
            // Legacy/normalized names
            | "POSITION_LOOK" | "FLYING_POSITION_LOOK" => {
                serde_json::from_value(self.data.clone())
                    .map(ParsedPacket::PositionLook)
                    .unwrap_or(ParsedPacket::Unknown(self.packet_type.clone()))
            }
            // PacketEvents name for bare flying packet
            "PLAYER_FLYING"
            // Legacy/normalized name
            | "FLYING" => {
                serde_json::from_value(self.data.clone())
                    .map(ParsedPacket::Flying)
                    .unwrap_or(ParsedPacket::Unknown(self.packet_type.clone()))
            }
            // PacketEvents uses INTERACT_ENTITY in many protocol versions.
            "USE_ENTITY" | "INTERACT_ENTITY" => {
                serde_json::from_value(self.data.clone())
                    .map(ParsedPacket::UseEntity)
                    .unwrap_or(ParsedPacket::Unknown(self.packet_type.clone()))
            }
            "ARM_ANIMATION" | "ANIMATION" => {
                serde_json::from_value(self.data.clone())
                    .map(ParsedPacket::ArmAnimation)
                    .unwrap_or(ParsedPacket::Unknown(self.packet_type.clone()))
            }
            "ENTITY_VELOCITY" => {
                serde_json::from_value(self.data.clone())
                    .map(ParsedPacket::EntityVelocity)
                    .unwrap_or(ParsedPacket::Unknown(self.packet_type.clone()))
            }
            "BLOCK_DIG" | "PLAYER_DIGGING" => {
                serde_json::from_value(self.data.clone())
                    .map(ParsedPacket::BlockDig)
                    .unwrap_or(ParsedPacket::Unknown(self.packet_type.clone()))
            }
            "BLOCK_PLACE" | "PLAYER_BLOCK_PLACEMENT" | "USE_ITEM" => {
                serde_json::from_value(self.data.clone())
                    .map(ParsedPacket::BlockPlace)
                    .unwrap_or(ParsedPacket::Unknown(self.packet_type.clone()))
            }
            "ENTITY_ACTION" => {
                serde_json::from_value(self.data.clone())
                    .map(ParsedPacket::EntityAction)
                    .unwrap_or(ParsedPacket::Unknown(self.packet_type.clone()))
            }
            "HELD_ITEM_SLOT" => {
                serde_json::from_value(self.data.clone())
                    .map(ParsedPacket::HeldItemSlot)
                    .unwrap_or(ParsedPacket::Unknown(self.packet_type.clone()))
            }
            "ABILITIES" | "PLAYER_ABILITIES" => {
                serde_json::from_value(self.data.clone())
                    .map(ParsedPacket::Abilities)
                    .unwrap_or(ParsedPacket::Unknown(self.packet_type.clone()))
            }
            "STEER_VEHICLE" => {
                serde_json::from_value(self.data.clone())
                    .map(ParsedPacket::SteerVehicle)
                    .unwrap_or(ParsedPacket::Unknown(self.packet_type.clone()))
            }
            "KEEP_ALIVE" => {
                serde_json::from_value(self.data.clone())
                    .map(ParsedPacket::KeepAlive)
                    .unwrap_or(ParsedPacket::Unknown(self.packet_type.clone()))
            }
            "WINDOW_CLICK" => {
                serde_json::from_value(self.data.clone())
                    .map(ParsedPacket::WindowClick)
                    .unwrap_or(ParsedPacket::Unknown(self.packet_type.clone()))
            }
            _ => ParsedPacket::Unknown(self.packet_type.clone()),
        }
    }
}

