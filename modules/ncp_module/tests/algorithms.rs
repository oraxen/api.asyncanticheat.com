use async_anticheat_ncp_module::{
    process_combat_events, process_movement_events, yaw_difference, CombatEvent, FightConfig, MovementEvent,
    MovingConfig, PlayerState,
};
use pretty_assertions::assert_eq;
use uuid::Uuid;

#[test]
fn yaw_difference_wraps() {
    assert_eq!(yaw_difference(10.0, 350.0), 20.0);
    assert_eq!(yaw_difference(350.0, 10.0), 20.0);
    assert_eq!(yaw_difference(0.0, 180.0), 180.0);
}

#[test]
fn fight_speed_flags_high_aps() {
    let player = Uuid::new_v4();
    let mut st = PlayerState::default();
    let cfg = FightConfig {
        speed_limit_aps: 4.0,
        speed_window_ms: 1000,
        ..FightConfig::default()
    };

    // 8 attacks in 350ms-ish.
    let events: Vec<CombatEvent> = (0..8)
        .map(|i| CombatEvent {
            ts: 1_000 + i * 50,
            player_uuid: player,
            entity_id: 1,
            player_x: Some(0.0),
            player_y: Some(64.0),
            player_z: Some(0.0),
            player_yaw: Some(0.0),
            player_pitch: Some(0.0),
            dt_ms: None,
            target_switched: None,
            yaw_diff: None,
            reach_distance: None,
            aim_off: None,
        })
        .collect();

    let findings = process_combat_events(&cfg, &mut st, player, &events, Some("k"));
    assert!(!findings.is_empty());
    assert!(findings.iter().any(|f| f.detector_name == "ncp_fight_speed"));
    assert!(st.fight_speed.vl > 0.0);
}

#[test]
fn fight_angle_flags_switching_yaw() {
    let player = Uuid::new_v4();
    let mut st = PlayerState::default();
    let cfg = FightConfig {
        angle_threshold: 10.0, // low threshold for test
        angle_max_window_ms: 1000,
        ..FightConfig::default()
    };

    let events = vec![
        CombatEvent {
            ts: 1_000,
            player_uuid: player,
            entity_id: 1,
            player_x: Some(0.0),
            player_y: Some(64.0),
            player_z: Some(0.0),
            player_yaw: Some(0.0),
            player_pitch: Some(0.0),
            dt_ms: None,
            target_switched: Some(false),
            yaw_diff: None,
            reach_distance: None,
            aim_off: None,
        },
        CombatEvent {
            ts: 1_060,
            player_uuid: player,
            entity_id: 2,
            player_x: Some(0.01),
            player_y: Some(64.0),
            player_z: Some(0.01),
            player_yaw: Some(120.0),
            player_pitch: Some(0.0),
            dt_ms: Some(60.0),
            target_switched: Some(true),
            yaw_diff: Some(120.0),
            reach_distance: None,
            aim_off: None,
        },
        CombatEvent {
            ts: 1_120,
            player_uuid: player,
            entity_id: 1,
            player_x: Some(0.02),
            player_y: Some(64.0),
            player_z: Some(0.02),
            player_yaw: Some(240.0),
            player_pitch: Some(0.0),
            dt_ms: Some(60.0),
            target_switched: Some(true),
            yaw_diff: Some(120.0),
            reach_distance: None,
            aim_off: None,
        },
    ];

    let findings = process_combat_events(&cfg, &mut st, player, &events, Some("k"));
    assert!(findings.iter().any(|f| f.detector_name == "ncp_fight_angle"));
    assert!(st.fight_angle.vl > 0.0);
}

#[test]
fn moving_speed_flags() {
    let player = Uuid::new_v4();
    let mut st = PlayerState::default();
    let cfg = MovingConfig {
        speed_limit_bps: 10.0,
        ..MovingConfig::default()
    };

    let events = vec![MovementEvent {
        ts: 2_000,
        player_uuid: player,
        x: 0.0,
        y: 64.0,
        z: 0.0,
        dt_ms: Some(50.0),
        dx: Some(1.0),
        dy: Some(0.0),
        dz: Some(0.0),
        speed_bps: Some(40.0),
        on_ground: Some(true),
    }];

    let findings = process_movement_events(&cfg, &mut st, player, &events, Some("k"));
    assert!(findings.iter().any(|f| f.detector_name == "ncp_moving_speed_basic"));
    assert!(st.moving_basic.speed_vl > 0.0);
}

#[test]
fn fight_reach_flags_when_distance_too_large() {
    let player = Uuid::new_v4();
    let mut st = PlayerState::default();
    let cfg = FightConfig {
        reach_limit_blocks: 4.4,
        ..FightConfig::default()
    };

    let events = vec![CombatEvent {
        ts: 1_000,
        player_uuid: player,
        entity_id: 1,
        player_x: Some(0.0),
        player_y: Some(64.0),
        player_z: Some(0.0),
        player_yaw: Some(0.0),
        player_pitch: Some(0.0),
        dt_ms: None,
        target_switched: None,
        yaw_diff: None,
        reach_distance: Some(6.0),
        aim_off: None,
    }];

    let findings = process_combat_events(&cfg, &mut st, player, &events, Some("k"));
    assert!(findings.iter().any(|f| f.detector_name == "ncp_fight_reach"));
    assert!(st.fight_reach.vl > 0.0);
}

#[test]
fn fight_wrongturn_flags_invalid_pitch() {
    let player = Uuid::new_v4();
    let mut st = PlayerState::default();
    let cfg = FightConfig::default();

    let events = vec![CombatEvent {
        ts: 1_000,
        player_uuid: player,
        entity_id: 1,
        player_x: Some(0.0),
        player_y: Some(64.0),
        player_z: Some(0.0),
        player_yaw: Some(0.0),
        player_pitch: Some(120.0),
        dt_ms: None,
        target_switched: None,
        yaw_diff: None,
        reach_distance: None,
        aim_off: None,
    }];

    let findings = process_combat_events(&cfg, &mut st, player, &events, Some("k"));
    assert!(findings.iter().any(|f| f.detector_name == "ncp_fight_wrongturn"));
    assert!(st.fight_wrongturn.vl >= 1.0);
}

#[test]
fn fight_direction_flags_aim_off() {
    let player = Uuid::new_v4();
    let mut st = PlayerState::default();
    let cfg = FightConfig {
        direction_off_threshold: 0.1,
        ..FightConfig::default()
    };

    let events = vec![CombatEvent {
        ts: 1_000,
        player_uuid: player,
        entity_id: 1,
        player_x: Some(0.0),
        player_y: Some(64.0),
        player_z: Some(0.0),
        player_yaw: Some(0.0),
        player_pitch: Some(0.0),
        dt_ms: None,
        target_switched: None,
        yaw_diff: None,
        reach_distance: None,
        aim_off: Some(0.5),
    }];

    let findings = process_combat_events(&cfg, &mut st, player, &events, Some("k"));
    assert!(findings.iter().any(|f| f.detector_name == "ncp_fight_direction"));
    assert!(st.fight_direction.vl > 0.0);
}

#[test]
fn state_round_trip_json() {
    let mut st = PlayerState::default();
    st.fight_angle.vl = 12.34;
    let v = serde_json::to_value(&st).unwrap();
    let st2: PlayerState = serde_json::from_value(v).unwrap();
    assert_eq!(st2.fight_angle.vl, 12.34);
}


