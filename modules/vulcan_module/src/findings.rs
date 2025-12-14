//! Findings and Feature IDs for Vulcan module
//!
//! Based on Vulcan 2.9.7.8 check types

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Vulcan Feature IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureId {
    // ========================================================================
    // Combat Checks
    // ========================================================================
    
    // Aim checks
    AimA, AimB, AimC, AimD, AimE, AimF, AimG, AimH, AimI, AimK, 
    AimL, AimM, AimN, AimO, AimP, AimQ, AimR, AimS, AimU, AimW, AimX, AimY,
    
    // Auto Block
    AutoBlockA, AutoBlockB, AutoBlockC, AutoBlockD,
    
    // Auto Clicker
    AutoClickerA, AutoClickerB, AutoClickerC, AutoClickerD, AutoClickerE,
    AutoClickerF, AutoClickerG, AutoClickerH, AutoClickerI, AutoClickerJ,
    AutoClickerK, AutoClickerL, AutoClickerM, AutoClickerN, AutoClickerO,
    AutoClickerP, AutoClickerQ, AutoClickerR, AutoClickerS, AutoClickerT,
    
    // Criticals
    CriticalsA, CriticalsB,
    
    // Fast Bow
    FastBowA,
    
    // Hitbox
    HitboxA, HitboxB,
    
    // Kill Aura
    KillAuraA, KillAuraB, KillAuraC, KillAuraD, KillAuraJ, KillAuraK, KillAuraL,
    
    // Reach
    ReachA, ReachB,
    
    // Velocity
    VelocityA, VelocityB, VelocityC, VelocityD,

    // ========================================================================
    // Movement Checks
    // ========================================================================
    
    // Anti Levitation
    AntiLevitationA,
    
    // Boat Fly
    BoatFlyA, BoatFlyB, BoatFlyC,
    
    // Elytra
    ElytraA, ElytraB, ElytraC, ElytraF, ElytraG, ElytraI, ElytraK, ElytraL, ElytraM, ElytraN,
    
    // Entity Flight
    EntityFlightA, EntityFlightB,
    
    // Entity Speed
    EntitySpeedA,
    
    // Fast Climb
    FastClimbA,
    
    // Flight
    FlightA, FlightB, FlightC, FlightD, FlightE, FlightF,
    
    // Jesus
    JesusA, JesusB, JesusC, JesusD, JesusE,
    
    // Jump
    JumpA, JumpB, JumpC, JumpD,
    
    // Motion
    MotionA, MotionB, MotionC, MotionE,
    
    // No Saddle
    NoSaddleA,
    
    // No Slow
    NoSlowA, NoSlowB, NoSlowC,
    
    // Speed
    SpeedA, SpeedB, SpeedC, SpeedD, SpeedE,
    
    // Sprint
    SprintA,
    
    // Step
    StepA, StepC,
    
    // Strafe
    StrafeA,
    
    // VClip
    VClipA,
    
    // Wall Climb
    WallClimbA,

    // ========================================================================
    // Player Checks
    // ========================================================================
    
    // Air Place
    AirPlaceA,
    
    // Bad Packets (numbered and lettered)
    BadPackets5, BadPackets6, BadPackets8, BadPackets9,
    BadPacketsA, BadPacketsB, BadPacketsC, BadPacketsD, BadPacketsE, BadPacketsF,
    BadPacketsG, BadPacketsH, BadPacketsI, BadPacketsJ, BadPacketsK, BadPacketsM,
    BadPacketsN, BadPacketsO, BadPacketsP, BadPacketsQ, BadPacketsR, BadPacketsT,
    BadPacketsV, BadPacketsW, BadPacketsX, BadPacketsY, BadPacketsZ,
    
    // Baritone
    BaritoneA, BaritoneB,
    
    // Fast Break
    FastBreakA,
    
    // Fast Place
    FastPlaceA,
    
    // Ghost Hand
    GhostHandA,
    
    // Ground Spoof
    GroundSpoofA, GroundSpoofB, GroundSpoofC,
    
    // Improbable
    ImprobableA, ImprobableB, ImprobableC, ImprobableD, ImprobableE, ImprobableF,
    
    // Invalid
    InvalidA, InvalidB, InvalidC, InvalidE, InvalidF, InvalidI, InvalidJ,
    
    // Scaffold
    ScaffoldA, ScaffoldB, ScaffoldC, ScaffoldD, ScaffoldE, ScaffoldF,
    ScaffoldG, ScaffoldH, ScaffoldI, ScaffoldJ, ScaffoldK, ScaffoldM, ScaffoldN,
    
    // Timer
    TimerA, TimerD,
    
    // Tower
    TowerA,
}

impl FeatureId {
    /// Get the detector name string
    pub fn detector_name(&self) -> &'static str {
        match self {
            // Aim
            Self::AimA => "aim_a",
            Self::AimB => "aim_b",
            Self::AimC => "aim_c",
            Self::AimD => "aim_d",
            Self::AimE => "aim_e",
            Self::AimF => "aim_f",
            Self::AimG => "aim_g",
            Self::AimH => "aim_h",
            Self::AimI => "aim_i",
            Self::AimK => "aim_k",
            Self::AimL => "aim_l",
            Self::AimM => "aim_m",
            Self::AimN => "aim_n",
            Self::AimO => "aim_o",
            Self::AimP => "aim_p",
            Self::AimQ => "aim_q",
            Self::AimR => "aim_r",
            Self::AimS => "aim_s",
            Self::AimU => "aim_u",
            Self::AimW => "aim_w",
            Self::AimX => "aim_x",
            Self::AimY => "aim_y",
            
            // Auto Block
            Self::AutoBlockA => "autoblock_a",
            Self::AutoBlockB => "autoblock_b",
            Self::AutoBlockC => "autoblock_c",
            Self::AutoBlockD => "autoblock_d",
            
            // Auto Clicker
            Self::AutoClickerA => "autoclicker_a",
            Self::AutoClickerB => "autoclicker_b",
            Self::AutoClickerC => "autoclicker_c",
            Self::AutoClickerD => "autoclicker_d",
            Self::AutoClickerE => "autoclicker_e",
            Self::AutoClickerF => "autoclicker_f",
            Self::AutoClickerG => "autoclicker_g",
            Self::AutoClickerH => "autoclicker_h",
            Self::AutoClickerI => "autoclicker_i",
            Self::AutoClickerJ => "autoclicker_j",
            Self::AutoClickerK => "autoclicker_k",
            Self::AutoClickerL => "autoclicker_l",
            Self::AutoClickerM => "autoclicker_m",
            Self::AutoClickerN => "autoclicker_n",
            Self::AutoClickerO => "autoclicker_o",
            Self::AutoClickerP => "autoclicker_p",
            Self::AutoClickerQ => "autoclicker_q",
            Self::AutoClickerR => "autoclicker_r",
            Self::AutoClickerS => "autoclicker_s",
            Self::AutoClickerT => "autoclicker_t",
            
            // Criticals
            Self::CriticalsA => "criticals_a",
            Self::CriticalsB => "criticals_b",
            
            // Fast Bow
            Self::FastBowA => "fastbow_a",
            
            // Hitbox
            Self::HitboxA => "hitbox_a",
            Self::HitboxB => "hitbox_b",
            
            // Kill Aura
            Self::KillAuraA => "killaura_a",
            Self::KillAuraB => "killaura_b",
            Self::KillAuraC => "killaura_c",
            Self::KillAuraD => "killaura_d",
            Self::KillAuraJ => "killaura_j",
            Self::KillAuraK => "killaura_k",
            Self::KillAuraL => "killaura_l",
            
            // Reach
            Self::ReachA => "reach_a",
            Self::ReachB => "reach_b",
            
            // Velocity
            Self::VelocityA => "velocity_a",
            Self::VelocityB => "velocity_b",
            Self::VelocityC => "velocity_c",
            Self::VelocityD => "velocity_d",
            
            // Anti Levitation
            Self::AntiLevitationA => "antilevitation_a",
            
            // Boat Fly
            Self::BoatFlyA => "boatfly_a",
            Self::BoatFlyB => "boatfly_b",
            Self::BoatFlyC => "boatfly_c",
            
            // Elytra
            Self::ElytraA => "elytra_a",
            Self::ElytraB => "elytra_b",
            Self::ElytraC => "elytra_c",
            Self::ElytraF => "elytra_f",
            Self::ElytraG => "elytra_g",
            Self::ElytraI => "elytra_i",
            Self::ElytraK => "elytra_k",
            Self::ElytraL => "elytra_l",
            Self::ElytraM => "elytra_m",
            Self::ElytraN => "elytra_n",
            
            // Entity Flight
            Self::EntityFlightA => "entityflight_a",
            Self::EntityFlightB => "entityflight_b",
            
            // Entity Speed
            Self::EntitySpeedA => "entityspeed_a",
            
            // Fast Climb
            Self::FastClimbA => "fastclimb_a",
            
            // Flight
            Self::FlightA => "flight_a",
            Self::FlightB => "flight_b",
            Self::FlightC => "flight_c",
            Self::FlightD => "flight_d",
            Self::FlightE => "flight_e",
            Self::FlightF => "flight_f",
            
            // Jesus
            Self::JesusA => "jesus_a",
            Self::JesusB => "jesus_b",
            Self::JesusC => "jesus_c",
            Self::JesusD => "jesus_d",
            Self::JesusE => "jesus_e",
            
            // Jump
            Self::JumpA => "jump_a",
            Self::JumpB => "jump_b",
            Self::JumpC => "jump_c",
            Self::JumpD => "jump_d",
            
            // Motion
            Self::MotionA => "motion_a",
            Self::MotionB => "motion_b",
            Self::MotionC => "motion_c",
            Self::MotionE => "motion_e",
            
            // No Saddle
            Self::NoSaddleA => "nosaddle_a",
            
            // No Slow
            Self::NoSlowA => "noslow_a",
            Self::NoSlowB => "noslow_b",
            Self::NoSlowC => "noslow_c",
            
            // Speed
            Self::SpeedA => "speed_a",
            Self::SpeedB => "speed_b",
            Self::SpeedC => "speed_c",
            Self::SpeedD => "speed_d",
            Self::SpeedE => "speed_e",
            
            // Sprint
            Self::SprintA => "sprint_a",
            
            // Step
            Self::StepA => "step_a",
            Self::StepC => "step_c",
            
            // Strafe
            Self::StrafeA => "strafe_a",
            
            // VClip
            Self::VClipA => "vclip_a",
            
            // Wall Climb
            Self::WallClimbA => "wallclimb_a",
            
            // Air Place
            Self::AirPlaceA => "airplace_a",
            
            // Bad Packets
            Self::BadPackets5 => "badpackets_5",
            Self::BadPackets6 => "badpackets_6",
            Self::BadPackets8 => "badpackets_8",
            Self::BadPackets9 => "badpackets_9",
            Self::BadPacketsA => "badpackets_a",
            Self::BadPacketsB => "badpackets_b",
            Self::BadPacketsC => "badpackets_c",
            Self::BadPacketsD => "badpackets_d",
            Self::BadPacketsE => "badpackets_e",
            Self::BadPacketsF => "badpackets_f",
            Self::BadPacketsG => "badpackets_g",
            Self::BadPacketsH => "badpackets_h",
            Self::BadPacketsI => "badpackets_i",
            Self::BadPacketsJ => "badpackets_j",
            Self::BadPacketsK => "badpackets_k",
            Self::BadPacketsM => "badpackets_m",
            Self::BadPacketsN => "badpackets_n",
            Self::BadPacketsO => "badpackets_o",
            Self::BadPacketsP => "badpackets_p",
            Self::BadPacketsQ => "badpackets_q",
            Self::BadPacketsR => "badpackets_r",
            Self::BadPacketsT => "badpackets_t",
            Self::BadPacketsV => "badpackets_v",
            Self::BadPacketsW => "badpackets_w",
            Self::BadPacketsX => "badpackets_x",
            Self::BadPacketsY => "badpackets_y",
            Self::BadPacketsZ => "badpackets_z",
            
            // Baritone
            Self::BaritoneA => "baritone_a",
            Self::BaritoneB => "baritone_b",
            
            // Fast Break
            Self::FastBreakA => "fastbreak_a",
            
            // Fast Place
            Self::FastPlaceA => "fastplace_a",
            
            // Ghost Hand
            Self::GhostHandA => "ghosthand_a",
            
            // Ground Spoof
            Self::GroundSpoofA => "groundspoof_a",
            Self::GroundSpoofB => "groundspoof_b",
            Self::GroundSpoofC => "groundspoof_c",
            
            // Improbable
            Self::ImprobableA => "improbable_a",
            Self::ImprobableB => "improbable_b",
            Self::ImprobableC => "improbable_c",
            Self::ImprobableD => "improbable_d",
            Self::ImprobableE => "improbable_e",
            Self::ImprobableF => "improbable_f",
            
            // Invalid
            Self::InvalidA => "invalid_a",
            Self::InvalidB => "invalid_b",
            Self::InvalidC => "invalid_c",
            Self::InvalidE => "invalid_e",
            Self::InvalidF => "invalid_f",
            Self::InvalidI => "invalid_i",
            Self::InvalidJ => "invalid_j",
            
            // Scaffold
            Self::ScaffoldA => "scaffold_a",
            Self::ScaffoldB => "scaffold_b",
            Self::ScaffoldC => "scaffold_c",
            Self::ScaffoldD => "scaffold_d",
            Self::ScaffoldE => "scaffold_e",
            Self::ScaffoldF => "scaffold_f",
            Self::ScaffoldG => "scaffold_g",
            Self::ScaffoldH => "scaffold_h",
            Self::ScaffoldI => "scaffold_i",
            Self::ScaffoldJ => "scaffold_j",
            Self::ScaffoldK => "scaffold_k",
            Self::ScaffoldM => "scaffold_m",
            Self::ScaffoldN => "scaffold_n",
            
            // Timer
            Self::TimerA => "timer_a",
            Self::TimerD => "timer_d",
            
            // Tower
            Self::TowerA => "tower_a",
        }
    }

    /// Get the check category
    pub fn category(&self) -> &'static str {
        match self {
            Self::AimA | Self::AimB | Self::AimC | Self::AimD | Self::AimE | Self::AimF |
            Self::AimG | Self::AimH | Self::AimI | Self::AimK | Self::AimL | Self::AimM |
            Self::AimN | Self::AimO | Self::AimP | Self::AimQ | Self::AimR | Self::AimS |
            Self::AimU | Self::AimW | Self::AimX | Self::AimY => "aim",
            
            Self::AutoBlockA | Self::AutoBlockB | Self::AutoBlockC | Self::AutoBlockD => "autoblock",
            
            Self::AutoClickerA | Self::AutoClickerB | Self::AutoClickerC | Self::AutoClickerD |
            Self::AutoClickerE | Self::AutoClickerF | Self::AutoClickerG | Self::AutoClickerH |
            Self::AutoClickerI | Self::AutoClickerJ | Self::AutoClickerK | Self::AutoClickerL |
            Self::AutoClickerM | Self::AutoClickerN | Self::AutoClickerO | Self::AutoClickerP |
            Self::AutoClickerQ | Self::AutoClickerR | Self::AutoClickerS | Self::AutoClickerT => "autoclicker",
            
            Self::CriticalsA | Self::CriticalsB => "criticals",
            Self::FastBowA => "fastbow",
            Self::HitboxA | Self::HitboxB => "hitbox",
            Self::KillAuraA | Self::KillAuraB | Self::KillAuraC | Self::KillAuraD |
            Self::KillAuraJ | Self::KillAuraK | Self::KillAuraL => "killaura",
            Self::ReachA | Self::ReachB => "reach",
            Self::VelocityA | Self::VelocityB | Self::VelocityC | Self::VelocityD => "velocity",
            
            Self::AntiLevitationA => "antilevitation",
            Self::BoatFlyA | Self::BoatFlyB | Self::BoatFlyC => "boatfly",
            Self::ElytraA | Self::ElytraB | Self::ElytraC | Self::ElytraF | Self::ElytraG |
            Self::ElytraI | Self::ElytraK | Self::ElytraL | Self::ElytraM | Self::ElytraN => "elytra",
            Self::EntityFlightA | Self::EntityFlightB => "entityflight",
            Self::EntitySpeedA => "entityspeed",
            Self::FastClimbA => "fastclimb",
            Self::FlightA | Self::FlightB | Self::FlightC | Self::FlightD | Self::FlightE | Self::FlightF => "flight",
            Self::JesusA | Self::JesusB | Self::JesusC | Self::JesusD | Self::JesusE => "jesus",
            Self::JumpA | Self::JumpB | Self::JumpC | Self::JumpD => "jump",
            Self::MotionA | Self::MotionB | Self::MotionC | Self::MotionE => "motion",
            Self::NoSaddleA => "nosaddle",
            Self::NoSlowA | Self::NoSlowB | Self::NoSlowC => "noslow",
            Self::SpeedA | Self::SpeedB | Self::SpeedC | Self::SpeedD | Self::SpeedE => "speed",
            Self::SprintA => "sprint",
            Self::StepA | Self::StepC => "step",
            Self::StrafeA => "strafe",
            Self::VClipA => "vclip",
            Self::WallClimbA => "wallclimb",
            
            Self::AirPlaceA => "airplace",
            Self::BadPackets5 | Self::BadPackets6 | Self::BadPackets8 | Self::BadPackets9 |
            Self::BadPacketsA | Self::BadPacketsB | Self::BadPacketsC | Self::BadPacketsD |
            Self::BadPacketsE | Self::BadPacketsF | Self::BadPacketsG | Self::BadPacketsH |
            Self::BadPacketsI | Self::BadPacketsJ | Self::BadPacketsK | Self::BadPacketsM |
            Self::BadPacketsN | Self::BadPacketsO | Self::BadPacketsP | Self::BadPacketsQ |
            Self::BadPacketsR | Self::BadPacketsT | Self::BadPacketsV | Self::BadPacketsW |
            Self::BadPacketsX | Self::BadPacketsY | Self::BadPacketsZ => "badpackets",
            Self::BaritoneA | Self::BaritoneB => "baritone",
            Self::FastBreakA => "fastbreak",
            Self::FastPlaceA => "fastplace",
            Self::GhostHandA => "ghosthand",
            Self::GroundSpoofA | Self::GroundSpoofB | Self::GroundSpoofC => "groundspoof",
            Self::ImprobableA | Self::ImprobableB | Self::ImprobableC | Self::ImprobableD |
            Self::ImprobableE | Self::ImprobableF => "improbable",
            Self::InvalidA | Self::InvalidB | Self::InvalidC | Self::InvalidE |
            Self::InvalidF | Self::InvalidI | Self::InvalidJ => "invalid",
            Self::ScaffoldA | Self::ScaffoldB | Self::ScaffoldC | Self::ScaffoldD | Self::ScaffoldE |
            Self::ScaffoldF | Self::ScaffoldG | Self::ScaffoldH | Self::ScaffoldI | Self::ScaffoldJ |
            Self::ScaffoldK | Self::ScaffoldM | Self::ScaffoldN => "scaffold",
            Self::TimerA | Self::TimerD => "timer",
            Self::TowerA => "tower",
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::AimA => "Invalid pitch change (slope)",
            Self::AimB => "Invalid yaw change (modulo)",
            Self::AimC => "Repeated yaw values",
            Self::AimD => "Invalid pitch change",
            Self::AimE => "Invalid yaw change (ratio)",
            Self::AimF => "Invalid yaw change (straight)",
            Self::AimG => "Too large yaw change",
            Self::AimH => "Invalid sensitivity (negative)",
            Self::AimI => "Not constant rotations",
            Self::AimK => "Linear rotations",
            Self::AimL => "Switching directions too quickly",
            Self::AimM => "Too small yaw change",
            Self::AimN => "Too small yaw change",
            Self::AimO => "Too small pitch change",
            Self::AimP => "Large yaw acceleration",
            Self::AimQ => "GCD modulo bypass",
            Self::AimR => "Rotation analysis heuristic",
            Self::AimS => "Subtle aim modifications",
            Self::AimU => "GCD flaw detected",
            Self::AimW => "Rotation analysis heuristic",
            Self::AimX => "Rotation analysis heuristic",
            Self::AimY => "Generic rotation analysis",
            
            Self::AutoBlockA => "Attacked while sending BlockPlace",
            Self::AutoBlockB => "Attacked while sending BlockDig",
            Self::AutoBlockC => "Invalid attack order",
            Self::AutoBlockD => "Invalid attack order",
            
            Self::AutoClickerA => "Left clicking too quickly (CPS)",
            Self::AutoClickerB => "Too low standard deviation",
            Self::AutoClickerC => "Rounded CPS values",
            Self::AutoClickerD => "Too low skewness",
            Self::AutoClickerE => "Too low variance",
            Self::AutoClickerF => "Not enough distinct values",
            Self::AutoClickerG => "Too low outliers",
            Self::AutoClickerH => "Similar deviation values",
            Self::AutoClickerI => "Too low kurtosis",
            Self::AutoClickerJ => "Impossible consistency (range)",
            Self::AutoClickerK => "Similar average values",
            Self::AutoClickerL => "Similar kurtosis values",
            Self::AutoClickerM => "Similar variance values",
            Self::AutoClickerN => "Low deviation difference",
            Self::AutoClickerO => "Impossible CPS spike",
            Self::AutoClickerP => "Identical statistical values",
            Self::AutoClickerQ => "Too low average deviation",
            Self::AutoClickerR => "Impossible consistency",
            Self::AutoClickerS => "Too few distinct delays",
            Self::AutoClickerT => "Too low kurtosis",
            
            Self::CriticalsA => "Critical hit on ground",
            Self::CriticalsB => "Critical hit modulo pattern",
            
            Self::FastBowA => "Shooting bow too quickly",
            
            Self::HitboxA => "Attacked without looking at target (history)",
            Self::HitboxB => "Attacked without looking at target (simple)",
            
            Self::KillAuraA => "Post UseEntity packets",
            Self::KillAuraB => "Invalid acceleration",
            Self::KillAuraC => "Head snap detected",
            Self::KillAuraD => "Attacked two entities at once",
            Self::KillAuraJ => "Suspicious attack frequency",
            Self::KillAuraK => "Suspicious attack pattern",
            Self::KillAuraL => "Impossible strafe while attacking",
            
            Self::ReachA => "Hit from too far (history)",
            Self::ReachB => "Hit from too far (simple)",
            
            Self::VelocityA => "Vertical velocity modification",
            Self::VelocityB => "Horizontal velocity modification",
            Self::VelocityC => "Ignored vertical velocity",
            Self::VelocityD => "Horizontal velocity modification",
            
            Self::AntiLevitationA => "Ignored levitation effect",
            Self::BoatFlyA => "Moving upwards in boat",
            Self::BoatFlyB => "Moving too fast in boat",
            Self::BoatFlyC => "Hovering in boat",
            
            Self::ElytraA => "Gliding too fast horizontally",
            Self::ElytraB => "Invalid Y-axis change while gliding",
            Self::ElytraC => "Invalid acceleration while gliding",
            Self::ElytraF => "Elytra packets too fast",
            Self::ElytraG => "Invalid acceleration",
            Self::ElytraI => "Invalid motion",
            Self::ElytraK => "Accelerating while ascending",
            Self::ElytraL => "Moving up wrongly",
            Self::ElytraM => "Invalid ascension pattern",
            Self::ElytraN => "Moving too fast on ground",
            
            Self::EntityFlightA => "Ascending while riding",
            Self::EntityFlightB => "Hovering while riding",
            Self::EntitySpeedA => "Riding too fast",
            Self::FastClimbA => "Climbing too quickly",
            
            Self::FlightA => "Invalid Y movement (server prediction)",
            Self::FlightB => "Invalid Y movement (client prediction)",
            Self::FlightC => "Invalid Y ascension",
            Self::FlightD => "Invalid glide",
            Self::FlightE => "Hovering",
            Self::FlightF => "Invalid Y movement",
            
            Self::JesusA => "Walking on water (ground)",
            Self::JesusB => "Invalid Y motion in water",
            Self::JesusC => "Invalid Y change in water",
            Self::JesusD => "Jumping on water",
            Self::JesusE => "Moving too fast on water",
            
            Self::JumpA => "Invalid jump motion",
            Self::JumpB => "Invalid jump height",
            Self::JumpC => "Invalid jump motion",
            Self::JumpD => "Invalid jump motion",
            
            Self::MotionA => "Repeated vertical motions",
            Self::MotionB => "Inverse motion pattern",
            Self::MotionC => "Invalid jump motion",
            Self::MotionE => "Impossible movement values",
            
            Self::NoSaddleA => "Controlling entity without saddle",
            
            Self::NoSlowA => "Placing while using item",
            Self::NoSlowB => "Moving too fast on soul sand",
            Self::NoSlowC => "Moving too fast in web",
            
            Self::SpeedA => "Invalid friction",
            Self::SpeedB => "Moving too fast on ground",
            Self::SpeedC => "Moving too fast in air",
            Self::SpeedD => "Moving too fast on ground",
            Self::SpeedE => "Invalid friction (prediction)",
            
            Self::SprintA => "Invalid sprint direction",
            
            Self::StepA => "Invalid step height",
            Self::StepC => "Reverse step",
            
            Self::StrafeA => "Moving incorrectly in air",
            Self::VClipA => "Large vertical clip",
            Self::WallClimbA => "Climbing wall without ladder",
            
            Self::AirPlaceA => "Invalid block placement (no support)",
            
            Self::BadPackets5 => "Invalid block place (scaffold)",
            Self::BadPackets6 => "Invalid block dig (nuker)",
            Self::BadPackets8 => "Invalid block dig (nuker)",
            Self::BadPackets9 => "Hitting without swinging",
            Self::BadPacketsA => "Spoofed abilities",
            Self::BadPacketsB => "Flying packet flood (>20)",
            Self::BadPacketsC => "Impossible pitch",
            Self::BadPacketsD => "Spoofed respawn",
            Self::BadPacketsE => "Self-interaction",
            Self::BadPacketsF => "Invalid steer vehicle",
            Self::BadPacketsG => "Invalid hotbar slot",
            Self::BadPacketsH => "Invalid attack packet order",
            Self::BadPacketsI => "EntityAction while attacking",
            Self::BadPacketsJ => "HeldItemSlot while placing",
            Self::BadPacketsK => "EntityAction spam",
            Self::BadPacketsM => "Post BlockPlace",
            Self::BadPacketsN => "Post BlockDig",
            Self::BadPacketsO => "Negative hotbar slot",
            Self::BadPacketsP => "Post EntityAction",
            Self::BadPacketsQ => "Hotbar slot > 8",
            Self::BadPacketsR => "Post HeldItemSlot",
            Self::BadPacketsT => "Invalid KeepAlive",
            Self::BadPacketsV => "Steer null vehicle",
            Self::BadPacketsW => "Post WindowClick",
            Self::BadPacketsX => "Post ArmAnimation",
            Self::BadPacketsY => "NaN position",
            Self::BadPacketsZ => "Invalid spectate",
            
            Self::BaritoneA => "Baritone-like rotations",
            Self::BaritoneB => "Baritone-like large rotations",
            
            Self::FastBreakA => "Breaking blocks too quickly",
            Self::FastPlaceA => "Placing blocks too quickly",
            Self::GhostHandA => "Invalid bed break",
            
            Self::GroundSpoofA => "Spoofed ground value",
            Self::GroundSpoofB => "Spoofed ground value",
            Self::GroundSpoofC => "Spoofed ground value",
            
            Self::ImprobableA => "Too many combat violations",
            Self::ImprobableB => "Too many movement violations",
            Self::ImprobableC => "Too many player violations",
            Self::ImprobableD => "Too many autoclicker violations",
            Self::ImprobableE => "Too many total violations",
            Self::ImprobableF => "Too many scaffold violations",
            
            Self::InvalidA => "Invalid position",
            Self::InvalidB => "Invalid position",
            Self::InvalidC => "Too large Y movement",
            Self::InvalidE => "Too large X/Z movement",
            Self::InvalidF => "Impossible Y change",
            Self::InvalidI => "Invalid Y movement",
            Self::InvalidJ => "Moving too quickly",
            
            Self::ScaffoldA => "Interacted with bottom of block",
            Self::ScaffoldB => "Invalid interact",
            Self::ScaffoldC => "Sprinting while bridging",
            Self::ScaffoldD => "Invalid rotations",
            Self::ScaffoldE => "Invalid rotations",
            Self::ScaffoldF => "Invalid rotations (packet)",
            Self::ScaffoldG => "Bridging too quickly",
            Self::ScaffoldH => "Invalid rotations",
            Self::ScaffoldI => "Invalid acceleration",
            Self::ScaffoldJ => "Invalid pitch change",
            Self::ScaffoldK => "Bridging too quickly",
            Self::ScaffoldM => "Invalid block face (expand)",
            Self::ScaffoldN => "Invalid block face (expand)",
            
            Self::TimerA => "Increased game speed (average)",
            Self::TimerD => "Increased game speed (balance)",
            
            Self::TowerA => "Towering too quickly",
        }
    }
}

/// Finding severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

/// A finding/detection from Vulcan checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub player_uuid: Uuid,
    pub feature: FeatureId,
    pub severity: Severity,
    pub title: String,
    pub description: Option<String>,
    pub buffer: f64,
    pub vl: u32,
    pub should_punish: bool,
    pub timestamp_ms: i64,
    pub evidence: Option<serde_json::Value>,
}

impl Finding {
    pub fn new(
        player_uuid: Uuid,
        feature: FeatureId,
        buffer: f64,
        vl: u32,
        max_vl: u32,
        timestamp_ms: i64,
    ) -> Self {
        let should_punish = vl >= max_vl;
        let severity = Self::calculate_severity(vl, max_vl, should_punish);
        let title = format!("{} (VL: {})", feature.description(), vl);

        Self {
            player_uuid,
            feature,
            severity,
            title,
            description: None,
            buffer,
            vl,
            should_punish,
            timestamp_ms,
            evidence: None,
        }
    }

    fn calculate_severity(vl: u32, max_vl: u32, should_punish: bool) -> Severity {
        if should_punish {
            Severity::Critical
        } else if max_vl > 0 {
            let ratio = vl as f32 / max_vl as f32;
            if ratio >= 0.8 {
                Severity::High
            } else if ratio >= 0.5 {
                Severity::Medium
            } else if ratio >= 0.2 {
                Severity::Low
            } else {
                Severity::Info
            }
        } else {
            Severity::Medium
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_evidence(mut self, evidence: serde_json::Value) -> Self {
        self.evidence = Some(evidence);
        self
    }
}

