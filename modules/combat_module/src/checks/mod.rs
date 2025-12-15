//! Combat check implementations
//!
//! Categories:
//! - killaura: Multi-target attacks, post-attack timing
//! - aim: Rotation analysis, GCD, snapping, patterns
//! - autoclicker: CPS, timing statistics, variance analysis
//! - reach: Attack distance validation
//! - noswing: Attack without arm animation

pub mod killaura;
pub mod aim;
pub mod autoclicker;
pub mod reach;
pub mod noswing;

use crate::config::CombatConfig;
use crate::findings::Finding;
use crate::packets::ParsedPacket;
use crate::player_state::PlayerState;

pub use killaura::KillAuraCheck;
pub use aim::AimCheck;
pub use autoclicker::AutoClickerCheck;
pub use reach::ReachCheck;
pub use noswing::NoSwingCheck;

/// Combined combat checks processor
pub struct CombatChecks {
    killaura: KillAuraCheck,
    aim: AimCheck,
    autoclicker: AutoClickerCheck,
    reach: ReachCheck,
    noswing: NoSwingCheck,
}

impl CombatChecks {
    pub fn new(config: CombatConfig) -> Self {
        Self {
            killaura: KillAuraCheck::new(config.killaura.clone()),
            aim: AimCheck::new(config.aim.clone()),
            autoclicker: AutoClickerCheck::new(config.autoclicker.clone()),
            reach: ReachCheck::new(config.reach.clone()),
            noswing: NoSwingCheck::new(config.noswing.clone()),
        }
    }

    /// Process a packet through all combat checks
    pub fn process(
        &self,
        state: &mut PlayerState,
        packet: &ParsedPacket,
        timestamp_ms: i64,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Process through each check
        findings.extend(self.killaura.process(state, packet, timestamp_ms));
        findings.extend(self.aim.process(state, packet, timestamp_ms));
        findings.extend(self.autoclicker.process(state, packet, timestamp_ms));
        findings.extend(self.reach.process(state, packet, timestamp_ms));
        findings.extend(self.noswing.process(state, packet, timestamp_ms));

        // Update general combat state based on packet type
        match packet {
            ParsedPacket::UseEntity(use_entity) if use_entity.action == "ATTACK" => {
                state.combat.in_combat = true;
                state.combat.combat_ticks += 1;
                state.combat.last_combat_ms = timestamp_ms;
                state.combat.total_attacks += 1;
            }
            ParsedPacket::Look(_) | ParsedPacket::PositionLook(_) => {
                // Count combat ticks when in combat
                if state.combat.in_combat {
                    state.combat.combat_ticks += 1;
                }
                // Exit combat after 10 seconds of no attacks
                if state.combat.in_combat && timestamp_ms - state.combat.last_combat_ms > 10000 {
                    state.combat.in_combat = false;
                    state.combat.combat_ticks = 0;
                }
            }
            _ => {}
        }

        findings
    }
}
