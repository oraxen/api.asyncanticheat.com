use anyhow::Context;
use axum::{
    body::Bytes,
    extract::State,
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, io::BufRead, io::BufReader};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

use async_anticheat_ncp_module::{
    process_combat_events, process_movement_events, CombatEvent, FightConfig, Finding, MovementEvent, MovingConfig,
    PlayerState,
};

#[derive(Clone)]
struct AppState {
    api_base: String,
    module_callback_token: String,
    module_name: String,
    fight_cfg: FightConfig,
    moving_cfg: MovingConfig,
    http: reqwest::Client,
}

#[derive(Serialize)]
struct Health {
    ok: bool,
    name: String,
}

#[derive(Deserialize)]
struct BatchMeta {
    // We don't rely on meta for routing, but keep it for forward compatibility.
    transform: Option<String>,
}

#[derive(Serialize)]
struct PostFindingsRequest {
    server_id: String,
    session_id: Option<String>,
    batch_id: Option<Uuid>,
    findings: Vec<Finding>,
}

#[derive(Serialize)]
struct BatchGetPlayerStatesRequest {
    server_id: String,
    player_uuids: Vec<Uuid>,
    module_name: String,
}

#[derive(Deserialize)]
struct BatchPlayerStateRow {
    player_uuid: Uuid,
    state: Value,
    updated_at: String,
}

#[derive(Deserialize)]
struct BatchGetPlayerStatesResponse {
    ok: bool,
    states: Vec<BatchPlayerStateRow>,
}

#[derive(Serialize)]
struct BatchSetPlayerStatesRequest {
    server_id: String,
    module_name: String,
    states: Vec<PlayerStateEntry>,
}

#[derive(Serialize)]
struct PlayerStateEntry {
    player_uuid: Uuid,
    state: Value,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(4020);

    let api_base = std::env::var("API_BASE").unwrap_or_else(|_| "http://127.0.0.1:3002".to_string());
    let module_callback_token = std::env::var("MODULE_CALLBACK_TOKEN").unwrap_or_default();
    let module_name = std::env::var("MODULE_NAME").unwrap_or_else(|_| "ncp_module".to_string());

    // Fight config
    let mut fight_cfg = FightConfig::default();
    if let Some(v) = std::env::var("FIGHT_ANGLE_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
    {
        fight_cfg.angle_threshold = v;
    }
    if let Some(v) = std::env::var("FIGHT_SPEED_LIMIT_APS")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
    {
        fight_cfg.speed_limit_aps = v;
    }
    if let Some(v) = std::env::var("FIGHT_REACH_LIMIT_BLOCKS")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
    {
        fight_cfg.reach_limit_blocks = v;
    }
    if let Some(v) = std::env::var("FIGHT_DIRECTION_OFF_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
    {
        fight_cfg.direction_off_threshold = v;
    }

    // Moving config
    let mut moving_cfg = MovingConfig::default();
    if let Some(v) = std::env::var("MOVING_SPEED_LIMIT_BPS")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
    {
        moving_cfg.speed_limit_bps = v;
    }

    if module_callback_token.is_empty() {
        tracing::warn!("MODULE_CALLBACK_TOKEN is empty; callbacks/state will be rejected by the API.");
    }

    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let state = AppState {
        api_base,
        module_callback_token,
        module_name,
        fight_cfg,
        moving_cfg,
        http,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/ingest", post(ingest))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", host, port).parse()?;
    tracing::info!("ncp_module listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

async fn health(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        ok: true,
        name: state.module_name,
    })
}

async fn ingest(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, (axum::http::StatusCode, Json<Value>)> {
    let server_id = headers
        .get("x-server-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    if server_id.is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error":"missing x-server-id"})),
        ));
    }
    let session_id = headers
        .get("x-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let batch_id = headers
        .get("x-batch-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok());
    let s3_key = headers
        .get("x-s3-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let transform = headers
        .get("x-transform")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    // --- Streaming decompress + NDJSON parse ---
    let decoder = GzDecoder::new(body.as_ref());
    let mut reader = BufReader::new(decoder);
    let mut line = String::new();

    // First line: meta (optional, ignore on failure).
    line.clear();
    if reader.read_line(&mut line).map_err(map_bad_gzip)? == 0 {
        return Ok(Json(serde_json::json!({"ok": true, "note": "empty batch"})));
    }
    let _meta: BatchMeta = serde_json::from_str(line.trim()).unwrap_or(BatchMeta { transform: None });

    // Collect events + player set (single pass).
    let mut player_ids: Vec<Uuid> = Vec::new();
    let mut seen: HashMap<Uuid, ()> = HashMap::new();

    let mut combat_by_player: HashMap<Uuid, Vec<CombatEvent>> = HashMap::new();
    let mut move_by_player: HashMap<Uuid, Vec<MovementEvent>> = HashMap::new();

    while {
        line.clear();
        reader.read_line(&mut line).map_err(map_bad_gzip)?
    } != 0
    {
        let l = line.trim_end_matches(&['\n', '\r'][..]);
        if l.is_empty() {
            continue;
        }

        if transform.eq_ignore_ascii_case("combat_events_v1_ndjson_gz")
            || transform.eq_ignore_ascii_case("ncp_fight_v1_ndjson_gz")
        {
            if let Ok(ev) = parse_combat_event(l) {
                if seen.insert(ev.player_uuid, ()).is_none() {
                    player_ids.push(ev.player_uuid);
                }
                combat_by_player.entry(ev.player_uuid).or_default().push(ev);
            }
        } else if transform.eq_ignore_ascii_case("movement_events_v1_ndjson_gz") {
            if let Ok(ev) = parse_movement_event(l) {
                if seen.insert(ev.player_uuid, ()).is_none() {
                    player_ids.push(ev.player_uuid);
                }
                move_by_player.entry(ev.player_uuid).or_default().push(ev);
            }
        } else {
            // Ignore unsupported transforms to keep this module focused and fast.
            continue;
        }
    }

    if player_ids.is_empty() {
        return Ok(Json(serde_json::json!({"ok": true, "processed_players": 0})));
    }

    // --- Batch get state ---
    let mut state_map = batch_get_states(&state, &server_id, &player_ids)
        .await
        .map_err(map_bad_gateway("state batch-get failed"))?;

    // --- Run checks ---
    let mut findings: Vec<Finding> = Vec::new();
    let evidence_s3_key = s3_key.as_deref();

    for player_uuid in &player_ids {
        let st = state_map.entry(*player_uuid).or_insert_with(PlayerState::default);

        if transform.eq_ignore_ascii_case("combat_events_v1_ndjson_gz")
            || transform.eq_ignore_ascii_case("ncp_fight_v1_ndjson_gz")
        {
            if let Some(events) = combat_by_player.get(player_uuid) {
                findings.extend(process_combat_events(
                    &state.fight_cfg,
                    st,
                    *player_uuid,
                    events,
                    evidence_s3_key,
                ));
            }
        } else if transform.eq_ignore_ascii_case("movement_events_v1_ndjson_gz") {
            if let Some(events) = move_by_player.get(player_uuid) {
                findings.extend(process_movement_events(
                    &state.moving_cfg,
                    st,
                    *player_uuid,
                    events,
                    evidence_s3_key,
                ));
            }
        }
    }

    // --- Batch set state (always, because decay + windows matter) ---
    batch_set_states(&state, &server_id, &state_map)
        .await
        .map_err(map_bad_gateway("state batch-set failed"))?;

    // --- Post findings (best-effort; if it fails we still 502 to signal module pipeline) ---
    if !findings.is_empty() {
        post_findings(&state, &server_id, session_id.as_deref(), batch_id, findings)
            .await
            .map_err(map_bad_gateway("post findings failed"))?;
    }

    Ok(Json(serde_json::json!({
        "ok": true,
        "server_id": server_id,
        "session_id": session_id,
        "batch_id": batch_id,
        "transform": transform,
        "processed_players": player_ids.len()
    })))
}

fn map_bad_gzip<E: std::fmt::Display>(e: E) -> (axum::http::StatusCode, Json<Value>) {
    (
        axum::http::StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": format!("gzip/ndjson read failed: {}", e)})),
    )
}

fn map_bad_gateway(msg: &'static str) -> impl Fn(anyhow::Error) -> (axum::http::StatusCode, Json<Value>) {
    move |e| {
        (
            axum::http::StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": msg, "detail": e.to_string()})),
        )
    }
}

// =============================================================================
// Parsing (fast, struct-based)
// =============================================================================

#[derive(Deserialize)]
struct CombatEventLine {
    ts: u64,
    uuid: String,
    entity_id: i64,
    #[serde(default)]
    player_x: Option<f64>,
    #[serde(default)]
    player_y: Option<f64>,
    #[serde(default)]
    player_z: Option<f64>,
    #[serde(default)]
    player_yaw: Option<f64>,
    #[serde(default)]
    player_pitch: Option<f64>,
    #[serde(default)]
    dt_ms: Option<f64>,
    #[serde(default)]
    target_switched: Option<bool>,
    #[serde(default)]
    yaw_diff: Option<f64>,
    #[serde(default)]
    reach_distance: Option<f64>,
    #[serde(default)]
    aim_off: Option<f64>,
    #[serde(default)]
    had_swing: Option<bool>,
}

fn parse_combat_event(line: &str) -> anyhow::Result<CombatEvent> {
    let ev: CombatEventLine = serde_json::from_str(line)?;
    let player_uuid = Uuid::parse_str(&ev.uuid)?;
    Ok(CombatEvent {
        ts: ev.ts,
        player_uuid,
        entity_id: ev.entity_id,
        player_x: ev.player_x,
        player_y: ev.player_y,
        player_z: ev.player_z,
        player_yaw: ev.player_yaw,
        player_pitch: ev.player_pitch,
        dt_ms: ev.dt_ms,
        target_switched: ev.target_switched,
        yaw_diff: ev.yaw_diff,
        reach_distance: ev.reach_distance,
        aim_off: ev.aim_off,
        had_swing: ev.had_swing,
    })
}

#[derive(Deserialize)]
struct MovementEventLine {
    ts: u64,
    uuid: String,
    x: f64,
    y: f64,
    z: f64,
    #[serde(default)]
    dt_ms: Option<f64>,
    #[serde(default)]
    dx: Option<f64>,
    #[serde(default)]
    dy: Option<f64>,
    #[serde(default)]
    dz: Option<f64>,
    #[serde(default)]
    speed_bps: Option<f64>,
    #[serde(default)]
    on_ground: Option<bool>,
}

fn parse_movement_event(line: &str) -> anyhow::Result<MovementEvent> {
    let ev: MovementEventLine = serde_json::from_str(line)?;
    let player_uuid = Uuid::parse_str(&ev.uuid)?;
    Ok(MovementEvent {
        ts: ev.ts,
        player_uuid,
        x: ev.x,
        y: ev.y,
        z: ev.z,
        dt_ms: ev.dt_ms,
        dx: ev.dx,
        dy: ev.dy,
        dz: ev.dz,
        speed_bps: ev.speed_bps,
        on_ground: ev.on_ground,
    })
}

// =============================================================================
// API calls (state + findings)
// =============================================================================

async fn batch_get_states(
    state: &AppState,
    server_id: &str,
    player_uuids: &[Uuid],
) -> anyhow::Result<HashMap<Uuid, PlayerState>> {
    let url = format!(
        "{}/callbacks/player-states/batch-get",
        state.api_base.trim_end_matches('/')
    );
    let req = BatchGetPlayerStatesRequest {
        server_id: server_id.to_string(),
        player_uuids: player_uuids.to_vec(),
        module_name: state.module_name.clone(),
    };
    let resp = state
        .http
        .post(url)
        .header("authorization", format!("Bearer {}", state.module_callback_token))
        .json(&req)
        .send()
        .await
        .context("batch-get request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("batch-get http {}", resp.status());
    }

    let body: BatchGetPlayerStatesResponse = resp.json().await.context("batch-get parse failed")?;
    if !body.ok {
        anyhow::bail!("batch-get returned ok=false");
    }

    let mut out = HashMap::new();
    for row in body.states {
        let st: PlayerState = serde_json::from_value(row.state).unwrap_or_default();
        out.insert(row.player_uuid, st);
    }
    Ok(out)
}

async fn batch_set_states(
    state: &AppState,
    server_id: &str,
    states: &HashMap<Uuid, PlayerState>,
) -> anyhow::Result<()> {
    let url = format!(
        "{}/callbacks/player-states/batch-set",
        state.api_base.trim_end_matches('/')
    );
    let mut entries = Vec::with_capacity(states.len());
    for (uuid, st) in states {
        entries.push(PlayerStateEntry {
            player_uuid: *uuid,
            state: serde_json::to_value(st).unwrap_or_else(|_| serde_json::json!({})),
        });
    }
    let req = BatchSetPlayerStatesRequest {
        server_id: server_id.to_string(),
        module_name: state.module_name.clone(),
        states: entries,
    };
    let resp = state
        .http
        .post(url)
        .header("authorization", format!("Bearer {}", state.module_callback_token))
        .json(&req)
        .send()
        .await
        .context("batch-set request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("batch-set http {}", resp.status());
    }
    Ok(())
}

async fn post_findings(
    state: &AppState,
    server_id: &str,
    session_id: Option<&str>,
    batch_id: Option<Uuid>,
    findings: Vec<Finding>,
) -> anyhow::Result<()> {
    let url = format!(
        "{}/callbacks/findings",
        state.api_base.trim_end_matches('/')
    );
    let req = PostFindingsRequest {
        server_id: server_id.to_string(),
        session_id: session_id.map(|s| s.to_string()),
        batch_id,
        findings,
    };
    let resp = state
        .http
        .post(url)
        .header("authorization", format!("Bearer {}", state.module_callback_token))
        .json(&req)
        .send()
        .await
        .context("post findings request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("post findings http {}", resp.status());
    }
    Ok(())
}


