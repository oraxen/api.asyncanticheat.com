//! Delays check (cm_0) - Actions performed faster than normally possible
//!
//! Detects:
//! - Fast block breaking
//! - Fast block placing
//! - Fast item use (eating, drinking)
//! - Fast bow shooting
//! - Fast regeneration
//! - Fast sneak toggle
//! - Fast bow release
//! - Break delay violations

use crate::config::DelaysConfig;
use crate::findings::{FeatureId, Finding};
use crate::packets::{BlockDigPacket, BlockPlacePacket, ParsedPacket, SneakPacket};
use crate::player_state::PlayerState;

/// Expected delays in milliseconds (based on vanilla Minecraft)
const BLOCK_BREAK_MIN_MS: i64 = 50;    // Minimum time between break start and end
const BLOCK_PLACE_MIN_MS: i64 = 50;    // Minimum time between placements
const ITEM_USE_MIN_MS: i64 = 1600;     // ~32 ticks for eating
const BOW_CHARGE_MIN_MS: i64 = 200;    // Minimum bow charge time
const REGEN_MIN_MS: i64 = 500;         // Minimum between regen ticks
const SNEAK_MIN_MS: i64 = 50;          // Minimum sneak toggle time
const BOW_RELEASE_MIN_MS: i64 = 100;   // Minimum bow release time

pub struct DelaysCheck {
    config: DelaysConfig,
}

impl DelaysCheck {
    pub fn new(config: DelaysConfig) -> Self {
        Self { config }
    }

    /// Process a packet and return any findings
    pub fn process(
        &self,
        state: &mut PlayerState,
        packet: &ParsedPacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        if !self.config.enabled {
            return Vec::new();
        }

        let mut findings = Vec::new();

        match packet {
            ParsedPacket::BlockDig(dig) => {
                if self.config.fast_break || self.config.break_delay {
                    findings.extend(self.check_block_dig(state, dig, timestamp_ms));
                }
            }
            ParsedPacket::BlockPlace(place) => {
                if self.config.fast_place {
                    findings.extend(self.check_block_place(state, place, timestamp_ms));
                }
            }
            ParsedPacket::ItemUse(_) => {
                if self.config.fast_use {
                    findings.extend(self.check_item_use(state, timestamp_ms));
                }
            }
            ParsedPacket::Sneak(sneak) => {
                if self.config.fast_sneak {
                    findings.extend(self.check_sneak(state, sneak, timestamp_ms));
                }
            }
            _ => {}
        }

        findings
    }

    fn check_block_dig(
        &self,
        state: &mut PlayerState,
        dig: &BlockDigPacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        match dig.status.as_str() {
            "START_DESTROY" | "START_DIGGING" => {
                state.delays.last_break_start_ms = timestamp_ms;
                state.delays.breaking_block = Some((dig.x, dig.y, dig.z));
            }
            "STOP_DESTROY" | "STOP_DIGGING" => {
                if state.delays.last_break_start_ms > 0 {
                    let break_time = timestamp_ms - state.delays.last_break_start_ms;

                    // Check for impossibly fast break
                    if break_time > 0 && break_time < BLOCK_BREAK_MIN_MS && self.config.fast_break {
                        let ratio = BLOCK_BREAK_MIN_MS as f32 / break_time as f32;
                        let mitigated = state.delays.vl.update(ratio - 1.0, timestamp_ms);

                        if ratio > 1.5 {
                            findings.push(
                                Finding::new(
                                    state.player_uuid,
                                    FeatureId::AacDelaysBreak,
                                    ratio,
                                    state.delays.vl.get(),
                                    mitigated,
                                    timestamp_ms,
                                )
                                .with_description(format!(
                                    "Block broken in {}ms (expected >= {}ms)",
                                    break_time, BLOCK_BREAK_MIN_MS
                                )),
                            );
                        }
                    }

                    // Check break delay (time since last break)
                    if state.delays.last_break_end_ms > 0 && self.config.break_delay {
                        let delay = timestamp_ms - state.delays.last_break_end_ms;
                        if delay > 0 && delay < BLOCK_BREAK_MIN_MS {
                            let ratio = BLOCK_BREAK_MIN_MS as f32 / delay as f32;
                            let mitigated = state.delays.vl.update(ratio - 1.0, timestamp_ms);

                            if ratio > 1.5 {
                                findings.push(Finding::new(
                                    state.player_uuid,
                                    FeatureId::AacDelaysBreakdelay,
                                    ratio,
                                    state.delays.vl.get(),
                                    mitigated,
                                    timestamp_ms,
                                ));
                            }
                        }
                    }

                    // Avoid moving backwards on out-of-order timestamps
                    if timestamp_ms > state.delays.last_break_end_ms {
                        state.delays.last_break_end_ms = timestamp_ms;
                    }
                }
                state.delays.breaking_block = None;
            }
            "ABORT_DESTROY" | "ABORT_DIGGING" => {
                state.delays.breaking_block = None;
            }
            _ => {}
        }

        findings
    }

    fn check_block_place(
        &self,
        state: &mut PlayerState,
        _place: &BlockPlacePacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        if state.delays.last_place_ms > 0 {
            let delay = timestamp_ms - state.delays.last_place_ms;

            if delay > 0 && delay < BLOCK_PLACE_MIN_MS {
                let ratio = BLOCK_PLACE_MIN_MS as f32 / delay as f32;
                let mitigated = state.delays.vl.update(ratio - 1.0, timestamp_ms);

                if ratio > 2.0 {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::AacDelaysPlace,
                            ratio,
                            state.delays.vl.get(),
                            mitigated,
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "Block placed {}ms after last (expected >= {}ms)",
                            delay, BLOCK_PLACE_MIN_MS
                        )),
                    );
                }
            }
        }

        // Avoid moving backwards on out-of-order timestamps
        if timestamp_ms > state.delays.last_place_ms {
            state.delays.last_place_ms = timestamp_ms;
        }
        findings
    }

    fn check_item_use(&self, state: &mut PlayerState, timestamp_ms: i64) -> Vec<Finding> {
        let mut findings = Vec::new();

        if state.delays.last_use_ms > 0 {
            let delay = timestamp_ms - state.delays.last_use_ms;

            // Item use (eating) should take ~32 ticks = 1600ms
            if delay > 0 && delay < ITEM_USE_MIN_MS {
                let ratio = ITEM_USE_MIN_MS as f32 / delay as f32;
                let mitigated = state.delays.vl.update(ratio - 1.0, timestamp_ms);

                if ratio > 1.5 {
                    findings.push(
                        Finding::new(
                            state.player_uuid,
                            FeatureId::AacDelaysConsume,
                            ratio,
                            state.delays.vl.get(),
                            mitigated,
                            timestamp_ms,
                        )
                        .with_description(format!(
                            "Item used in {}ms (expected >= {}ms)",
                            delay, ITEM_USE_MIN_MS
                        )),
                    );
                }
            }
        }

        // Avoid moving backwards on out-of-order timestamps
        if timestamp_ms > state.delays.last_use_ms {
            state.delays.last_use_ms = timestamp_ms;
        }
        findings
    }

    fn check_sneak(
        &self,
        state: &mut PlayerState,
        sneak: &SneakPacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Only check unsneak (end sneak)
        if !sneak.sneaking && state.delays.last_sneak_ms > 0 {
            let delay = timestamp_ms - state.delays.last_sneak_ms;

            if delay > 0 && delay < SNEAK_MIN_MS {
                let ratio = SNEAK_MIN_MS as f32 / delay as f32;
                let mitigated = state.delays.vl.update(ratio - 1.0, timestamp_ms);

                if ratio > 2.0 {
                    findings.push(Finding::new(
                        state.player_uuid,
                        FeatureId::AacDelaysUnsneak,
                        ratio,
                        state.delays.vl.get(),
                        mitigated,
                        timestamp_ms,
                    ));
                }
            }
        }

        if sneak.sneaking {
            // Avoid moving backwards on out-of-order timestamps
            if timestamp_ms > state.delays.last_sneak_ms {
                state.delays.last_sneak_ms = timestamp_ms;
            }
        }

        findings
    }
}

