//! Packet types for combat module

use serde::{Deserialize, Serialize};

/// Parsed packet types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParsedPacket {
    Look(LookPacket),
    PositionLook(PositionLookPacket),
    UseEntity(UseEntityPacket),
    ArmAnimation(ArmAnimationPacket),
    EntityVelocity(EntityVelocityPacket),
    Position(PositionPacket),
    EntityAction(EntityActionPacket),
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookPacket {
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionLookPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseEntityPacket {
    pub entity_id: i32,
    pub action: String,
    pub target_x: Option<f64>,
    pub target_y: Option<f64>,
    pub target_z: Option<f64>,
    pub hand: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmAnimationPacket {
    pub hand: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityVelocityPacket {
    pub entity_id: i32,
    pub velocity_x: f64,
    pub velocity_y: f64,
    pub velocity_z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityActionPacket {
    pub entity_id: i32,
    pub action: String,
    pub jump_boost: Option<i32>,
}

/// Location with position and rotation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

/// Parse a packet from JSON
pub fn parse_packet(json: &serde_json::Value) -> Option<ParsedPacket> {
    let packet_type = json.get("type")?.as_str()?;
    
    match packet_type {
        "LOOK" | "PLAYER_LOOK" => {
            Some(ParsedPacket::Look(LookPacket {
                yaw: json.get("yaw")?.as_f64()? as f32,
                pitch: json.get("pitch")?.as_f64()? as f32,
                on_ground: json.get("onGround").and_then(|v| v.as_bool()).unwrap_or(false),
            }))
        }
        "POSITION_LOOK" | "PLAYER_POSITION_LOOK" => {
            Some(ParsedPacket::PositionLook(PositionLookPacket {
                x: json.get("x")?.as_f64()?,
                y: json.get("y")?.as_f64()?,
                z: json.get("z")?.as_f64()?,
                yaw: json.get("yaw")?.as_f64()? as f32,
                pitch: json.get("pitch")?.as_f64()? as f32,
                on_ground: json.get("onGround").and_then(|v| v.as_bool()).unwrap_or(false),
            }))
        }
        "POSITION" | "PLAYER_POSITION" => {
            Some(ParsedPacket::Position(PositionPacket {
                x: json.get("x")?.as_f64()?,
                y: json.get("y")?.as_f64()?,
                z: json.get("z")?.as_f64()?,
                on_ground: json.get("onGround").and_then(|v| v.as_bool()).unwrap_or(false),
            }))
        }
        "USE_ENTITY" => {
            Some(ParsedPacket::UseEntity(UseEntityPacket {
                entity_id: json.get("entityId")?.as_i64()? as i32,
                action: json.get("action")?.as_str()?.to_string(),
                target_x: json.get("targetX").and_then(|v| v.as_f64()),
                target_y: json.get("targetY").and_then(|v| v.as_f64()),
                target_z: json.get("targetZ").and_then(|v| v.as_f64()),
                hand: json.get("hand").and_then(|v| v.as_str()).map(String::from),
            }))
        }
        "ARM_ANIMATION" | "ANIMATION" => {
            Some(ParsedPacket::ArmAnimation(ArmAnimationPacket {
                hand: json.get("hand").and_then(|v| v.as_str()).map(String::from),
            }))
        }
        "ENTITY_VELOCITY" => {
            Some(ParsedPacket::EntityVelocity(EntityVelocityPacket {
                entity_id: json.get("entityId")?.as_i64()? as i32,
                velocity_x: json.get("velocityX")?.as_f64()?,
                velocity_y: json.get("velocityY")?.as_f64()?,
                velocity_z: json.get("velocityZ")?.as_f64()?,
            }))
        }
        "ENTITY_ACTION" => {
            Some(ParsedPacket::EntityAction(EntityActionPacket {
                entity_id: json.get("entityId")?.as_i64()? as i32,
                action: json.get("action")?.as_str()?.to_string(),
                jump_boost: json.get("jumpBoost").and_then(|v| v.as_i64()).map(|v| v as i32),
            }))
        }
        _ => Some(ParsedPacket::Unknown(packet_type.to_string())),
    }
}
