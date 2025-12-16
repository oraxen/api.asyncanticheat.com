//! Packet types for player module

use serde::{Deserialize, Serialize};

/// Parsed packet types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParsedPacket {
    Look(LookPacket),
    PositionLook(PositionLookPacket),
    Position(PositionPacket),
    Flying(FlyingPacket),
    BlockPlace(BlockPlacePacket),
    BlockDig(BlockDigPacket),
    HeldItemSlot(HeldItemSlotPacket),
    Abilities(AbilitiesPacket),
    Sneak(SneakPacket),
    WindowClick(WindowClickPacket),
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
pub struct FlyingPacket {
    pub on_ground: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPlacePacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub face: i32,
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub cursor_z: f32,
    pub inside_block: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDigPacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub face: i32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeldItemSlotPacket {
    pub slot: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilitiesPacket {
    pub is_flying: bool,
    pub allow_flying: Option<bool>,
    pub creative_mode: Option<bool>,
    pub invulnerable: Option<bool>,
    /// True when player claims instant block breaking (creative mode ability)
    pub instant_break: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SneakPacket {
    pub sneaking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowClickPacket {
    pub window_id: i32,
    pub slot: i32,
    pub button: i32,
    pub mode: i32,
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
        "FLYING" | "PLAYER_FLYING" => {
            Some(ParsedPacket::Flying(FlyingPacket {
                on_ground: json.get("onGround").and_then(|v| v.as_bool()).unwrap_or(false),
            }))
        }
        "BLOCK_PLACE" | "USE_ITEM" => {
            Some(ParsedPacket::BlockPlace(BlockPlacePacket {
                x: json.get("x").or(json.get("blockX")).and_then(|v| v.as_i64())? as i32,
                y: json.get("y").or(json.get("blockY")).and_then(|v| v.as_i64())? as i32,
                z: json.get("z").or(json.get("blockZ")).and_then(|v| v.as_i64())? as i32,
                face: json.get("face").or(json.get("direction")).and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                cursor_x: json.get("cursorX").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
                cursor_y: json.get("cursorY").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
                cursor_z: json.get("cursorZ").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
                inside_block: json.get("insideBlock").and_then(|v| v.as_bool()),
            }))
        }
        "BLOCK_DIG" => {
            Some(ParsedPacket::BlockDig(BlockDigPacket {
                x: json.get("x").or(json.get("blockX")).and_then(|v| v.as_i64())? as i32,
                y: json.get("y").or(json.get("blockY")).and_then(|v| v.as_i64())? as i32,
                z: json.get("z").or(json.get("blockZ")).and_then(|v| v.as_i64())? as i32,
                face: json.get("face").or(json.get("direction")).and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                status: json.get("status").and_then(|v| v.as_str()).unwrap_or("START").to_string(),
            }))
        }
        "HELD_ITEM_SLOT" | "HELD_ITEM_CHANGE" => {
            Some(ParsedPacket::HeldItemSlot(HeldItemSlotPacket {
                slot: json.get("slot").and_then(|v| v.as_i64())? as i32,
            }))
        }
        "ABILITIES" | "PLAYER_ABILITIES" => {
            Some(ParsedPacket::Abilities(AbilitiesPacket {
                is_flying: json.get("isFlying").or(json.get("flying")).and_then(|v| v.as_bool()).unwrap_or(false),
                allow_flying: json.get("allowFlying").or(json.get("allow_flying")).and_then(|v| v.as_bool()),
                creative_mode: json.get("creativeMode").and_then(|v| v.as_bool()),
                invulnerable: json.get("invulnerable").and_then(|v| v.as_bool()),
                instant_break: json.get("instant_break").or(json.get("instantBreak")).and_then(|v| v.as_bool()),
            }))
        }
        "SNEAK" | "ENTITY_ACTION" if json.get("action").and_then(|v| v.as_str()).map(|s| s.contains("SNEAK")).unwrap_or(false) => {
            let action = json.get("action").and_then(|v| v.as_str()).unwrap_or("");
            Some(ParsedPacket::Sneak(SneakPacket {
                sneaking: action.contains("START") || action == "PRESS_SHIFT_KEY",
            }))
        }
        "ENTITY_ACTION" => {
            Some(ParsedPacket::EntityAction(EntityActionPacket {
                entity_id: json.get("entityId").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                action: json.get("action").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                jump_boost: json.get("jumpBoost").and_then(|v| v.as_i64()).map(|v| v as i32),
            }))
        }
        "WINDOW_CLICK" | "CLICK_WINDOW" => {
            Some(ParsedPacket::WindowClick(WindowClickPacket {
                window_id: json.get("windowId").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                slot: json.get("slot").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                button: json.get("button").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                mode: json.get("mode").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            }))
        }
        _ => Some(ParsedPacket::Unknown(packet_type.to_string())),
    }
}
