//! Player checks: Bad Packets, Scaffold, Fast Break, Fast Place, Invalid

use crate::config::VulcanConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

/// Player check thresholds
const MIN_PLACE_INTERVAL_MS: i64 = 50;
const MIN_BREAK_INTERVAL_MS: i64 = 50;
const MAX_HOTBAR_SLOT: i32 = 8;

pub struct PlayerChecks {
    config: VulcanConfig,
}

impl PlayerChecks {
    pub fn new(config: VulcanConfig) -> Self {
        Self { config }
    }

    pub fn process(
        &self,
        state: &mut PlayerState,
        packet: &ParsedPacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        match packet {
            ParsedPacket::BlockPlace(place) => {
                findings.extend(self.check_block_place(state, &place.face, timestamp_ms));
            }
            ParsedPacket::BlockDig(dig) => {
                findings.extend(self.check_block_dig(state, &dig.status, timestamp_ms));
            }
            ParsedPacket::HeldItemSlot(slot) => {
                findings.extend(self.check_hotbar(state, slot.slot, timestamp_ms));
            }
            ParsedPacket::Abilities(abilities) => {
                findings.extend(self.check_abilities(state, abilities, timestamp_ms));
            }
            ParsedPacket::PositionLook(pos) => {
                // Bad Packets C - Invalid pitch
                if pos.pitch > 90.0 || pos.pitch < -90.0 {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::BadPacketsC,
                            pos.pitch as f64,
                            1,
                            1,
                            timestamp_ms,
                        )
                        .with_description(format!("Invalid pitch: {:.1}° (valid: -90 to 90)", pos.pitch)),
                    );
                    state.player_violations += 1;
                }

                // Bad Packets Y - NaN position
                if pos.x.is_nan() || pos.y.is_nan() || pos.z.is_nan() {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::BadPacketsY,
                            1.0,
                            1,
                            1,
                            timestamp_ms,
                        )
                        .with_description("NaN position values".to_string()),
                    );
                    state.player_violations += 1;
                }

                // Invalid C - Too large Y movement
                if let Some(last_loc) = state.movement.last_location {
                    let y_diff = (pos.y - last_loc.y).abs();
                    if y_diff > 10.0 {
                        findings.push(
                            Finding::new(
                                state.player_uuid,
                                FeatureId::InvalidC,
                                y_diff,
                                1,
                                5,
                                timestamp_ms,
                            )
                            .with_description(format!("Y movement: {:.2} blocks", y_diff)),
                        );
                        state.player_violations += 1;
                    }

                    // Invalid E - Too large X/Z movement
                    let h_diff = ((pos.x - last_loc.x).powi(2) + (pos.z - last_loc.z).powi(2)).sqrt();
                    if h_diff > 10.0 {
                        findings.push(
                            Finding::new(
                                state.player_uuid,
                                FeatureId::InvalidE,
                                h_diff,
                                1,
                                5,
                                timestamp_ms,
                            )
                            .with_description(format!("Horizontal movement: {:.2} blocks", h_diff)),
                        );
                        state.player_violations += 1;
                    }
                }
            }
            ParsedPacket::Look(look) => {
                // Bad Packets C - Invalid pitch
                if look.pitch > 90.0 || look.pitch < -90.0 {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::BadPacketsC,
                            look.pitch as f64,
                            1,
                            1,
                            timestamp_ms,
                        )
                        .with_description(format!("Invalid pitch: {:.1}°", look.pitch)),
                    );
                    state.player_violations += 1;
                }
            }
            ParsedPacket::SteerVehicle(_) => {
                // Bad Packets V - Steer without vehicle
                if !state.movement.in_vehicle {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::BadPacketsV,
                            1.0,
                            1,
                            1,
                            timestamp_ms,
                        )
                        .with_description("SteerVehicle while not in vehicle".to_string()),
                    );
                    state.player_violations += 1;
                }
            }
            _ => {}
        }

        // Track last packet for post-packet checks
        state.badpackets.last_packet_type = format!("{:?}", packet);
        state.badpackets.last_packet_ms = timestamp_ms;

        findings
    }

    /// Block place checks (Fast Place, Scaffold)
    fn check_block_place(
        &self,
        state: &mut PlayerState,
        face: &str,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Fast Place A
        if self.config.fastplace.enabled && state.scaffold.last_place_ms > 0 {
            let interval = timestamp_ms - state.scaffold.last_place_ms;
            if interval < MIN_PLACE_INTERVAL_MS {
                findings.push(
                    Finding::new(
                        state.player_uuid,
                        FeatureId::FastPlaceA,
                        interval as f64,
                        1,
                        10,
                        timestamp_ms,
                    )
                    .with_description(format!("Fast place: {}ms (min: {}ms)", interval, MIN_PLACE_INTERVAL_MS)),
                );
                state.player_violations += 1;
            }
        }

        // Scaffold A - Interacted with bottom of block
        if self.config.scaffold.enabled && face == "DOWN" {
            // Check if player is above the block (bridging)
            if let Some(loc) = state.movement.last_location {
                if !loc.on_ground {
                    let flagged = state.scaffold.buffer.fail();
                    if flagged {
                        findings.push(
                            Finding::new(
                                state.player_uuid,
                                FeatureId::ScaffoldA,
                                state.scaffold.buffer.get(),
                                state.scaffold.buffer.vl(),
                                state.scaffold.buffer.max_vl(),
                                timestamp_ms,
                            )
                            .with_description("Placed block on bottom face while airborne".to_string()),
                        );
                        state.player_violations += 1;
                    }
                } else {
                    state.scaffold.buffer.pass();
                }
            }
        }

        // Scaffold C - Sprinting while scaffolding
        if self.config.scaffold.c.enabled && face == "DOWN" && state.movement.is_sprinting {
            findings.push(
                Finding::new(
                    state.player_uuid,
                    FeatureId::ScaffoldC,
                    1.0,
                    1,
                    5,
                    timestamp_ms,
                )
                .with_description("Sprinting while bridging".to_string()),
            );
            state.player_violations += 1;
        }

        state.scaffold.last_place_ms = timestamp_ms;
        state.scaffold.last_place_face = face.to_string();
        state.scaffold.place_count += 1;

        findings
    }

    /// Block dig checks (Fast Break)
    fn check_block_dig(
        &self,
        state: &mut PlayerState,
        status: &str,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Only check on STOP_DIGGING (block broken)
        if status == "STOP_DIGGING" || status == "STOP_DESTROY" {
            // Fast Break would need to track start/stop time per block
            // Simplified check for rapid consecutive breaks
            if state.badpackets.last_packet_type.contains("BlockDig") {
                let interval = timestamp_ms - state.badpackets.last_packet_ms;
                if interval < MIN_BREAK_INTERVAL_MS && self.config.fastbreak.enabled {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::FastBreakA,
                            interval as f64,
                            1,
                            10,
                            timestamp_ms,
                        )
                        .with_description(format!("Fast break: {}ms interval", interval)),
                    );
                    state.player_violations += 1;
                }
            }
        }

        findings
    }

    /// Hotbar checks
    fn check_hotbar(
        &self,
        state: &mut PlayerState,
        slot: i32,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Bad Packets O - Negative slot
        if slot < 0 {
            findings.push(
                Finding::new(
                    state.player_uuid,
                    FeatureId::BadPacketsO,
                    slot as f64,
                    1,
                    1,
                    timestamp_ms,
                )
                .with_description(format!("Negative hotbar slot: {}", slot)),
            );
            state.player_violations += 1;
        }

        // Bad Packets Q - Slot > 8
        if slot > MAX_HOTBAR_SLOT {
            findings.push(
                Finding::new(
                    state.player_uuid,
                    FeatureId::BadPacketsQ,
                    slot as f64,
                    1,
                    1,
                    timestamp_ms,
                )
                .with_description(format!("Hotbar slot {} > max {}", slot, MAX_HOTBAR_SLOT)),
            );
            state.player_violations += 1;
        }

        // Bad Packets G - Same slot
        if slot == state.badpackets.last_hotbar_slot {
            // Sending same slot multiple times is suspicious
            // But only flag if it happens repeatedly
        }

        state.badpackets.last_hotbar_slot = slot;
        findings
    }

    /// Abilities checks
    fn check_abilities(
        &self,
        state: &mut PlayerState,
        abilities: &crate::packets::AbilitiesPacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Bad Packets A - Spoofed abilities
        // Flying without permission
        if abilities.flying && !abilities.allow_flying {
            findings.push(
                Finding::new(
                    state.player_uuid,
                    FeatureId::BadPacketsA,
                    1.0,
                    1,
                    1,
                    timestamp_ms,
                )
                .with_description("Flying without allow_flying permission".to_string()),
            );
            state.player_violations += 1;
        }

        findings
    }
}

