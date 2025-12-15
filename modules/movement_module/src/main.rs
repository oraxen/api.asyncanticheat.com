//! Movement Module - Unified movement cheat detection
//!
//! This module provides detection for:
//! - Flight (Y prediction, sustained ascension, hover)
//! - Speed (horizontal speed, sneak/sprint)
//! - NoFall (invalid ground claims)
//! - Timer (packet rate manipulation)
//! - Step (climbing too high)
//! - GroundSpoof (claiming ground while falling)
//! - Velocity (knockback ignoring)
//! - NoSlow (moving too fast while sneaking/using items)

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

use movement_module::{
    MovementChecks, MovementConfig, Finding, ParsedPacket, PlayerState,
    packets::parse_packet,
};

#[derive(Clone)]
struct AppState {
    api_base: String,
    module_callback_token: String,
    module_name: String,
    config: MovementConfig,
    http: reqwest::Client,
}

#[derive(Serialize)]
struct Health {
    ok: bool,
    name: String,
    version: String,
}

#[derive(Serialize)]
struct PostFindingsRequest {
    server_id: String,
    session_id: Option<String>,
    batch_id: Option<Uuid>,
    findings: Vec<FindingDto>,
}

#[derive(Serialize)]
struct FindingDto {
    player_uuid: Option<Uuid>,
    detector_name: String,
    detector_version: Option<String>,
    severity: String,
    title: String,
    description: Option<String>,
    evidence_s3_key: Option<String>,
    evidence_json: Option<Value>,
}

impl From<Finding> for FindingDto {
    fn from(f: Finding) -> Self {
        Self {
            player_uuid: Some(f.player_uuid),
            detector_name: format!("movement_{}", f.feature_id.name()),
            detector_version: Some("1".to_string()),
            severity: if f.should_mitigate { "violation".to_string() } else { "warning".to_string() },
            title: f.feature_id.name().to_string(),
            description: f.description,
            evidence_s3_key: None,
            evidence_json: f.evidence,
        }
    }
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

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(4022);

    let api_base = std::env::var("API_BASE").unwrap_or_else(|_| "http://localhost:3002".to_string());
    let module_callback_token = std::env::var("MODULE_CALLBACK_TOKEN").unwrap_or_default();
    let module_name = std::env::var("MODULE_NAME").unwrap_or_else(|_| "movement_module".to_string());

    let config = MovementConfig::default();

    if module_callback_token.is_empty() {
        tracing::warn!("MODULE_CALLBACK_TOKEN is empty; callbacks will be rejected by the API.");
    }

    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let state = AppState {
        api_base,
        module_callback_token,
        module_name,
        config,
        http,
    };

    tracing::info!("Starting Movement Module v0.1.0");
    tracing::info!("API URL: {}", state.api_base);
    tracing::info!("Listening on: {}:{}", host, port);

    let app = Router::new()
        .route("/health", get(health))
        .route("/ingest", post(ingest))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", host, port).parse()?;
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

async fn health(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        ok: true,
        name: state.module_name,
        version: "0.1.0".to_string(),
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
            Json(serde_json::json!({"error": "missing x-server-id"})),
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

    // Decompress and parse
    let decoder = GzDecoder::new(body.as_ref());
    let mut reader = BufReader::new(decoder);
    let mut line = String::new();

    // Skip meta line
    line.clear();
    if reader.read_line(&mut line).map_err(map_bad_gzip)? == 0 {
        return Ok(Json(serde_json::json!({"ok": true, "note": "empty batch"})));
    }

    // Collect packets by player
    let mut packets_by_player: HashMap<Uuid, Vec<(i64, ParsedPacket)>> = HashMap::new();
    let mut player_ids: Vec<Uuid> = Vec::new();
    let mut seen: HashMap<Uuid, ()> = HashMap::new();

    while {
        line.clear();
        reader.read_line(&mut line).map_err(map_bad_gzip)?
    } != 0
    {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }

        if let Ok(json) = serde_json::from_str::<Value>(l) {
            let uuid_str = json.get("uuid").or_else(|| json.get("playerUuid")).and_then(|v| v.as_str());
            let timestamp = json.get("ts").or_else(|| json.get("timestamp")).and_then(|v| v.as_i64()).unwrap_or(0);

            if let Some(uuid_str) = uuid_str {
                if let Ok(player_uuid) = Uuid::parse_str(uuid_str) {
                    if seen.insert(player_uuid, ()).is_none() {
                        player_ids.push(player_uuid);
                    }

                    if let Some(packet) = parse_packet(&json) {
                        packets_by_player.entry(player_uuid).or_default().push((timestamp, packet));
                    }
                }
            }
        }
    }

    if player_ids.is_empty() {
        return Ok(Json(serde_json::json!({"ok": true, "processed_players": 0})));
    }

    // Batch get states
    let mut state_map = batch_get_states(&state, &server_id, &player_ids)
        .await
        .map_err(map_bad_gateway("state batch-get failed"))?;

    // Run checks
    let checks = MovementChecks::new(state.config.clone());
    let mut all_findings: Vec<Finding> = Vec::new();

    for player_uuid in &player_ids {
        let player_state = state_map
            .entry(*player_uuid)
            .or_insert_with(|| PlayerState::new(*player_uuid));

        if let Some(packets) = packets_by_player.get(player_uuid) {
            for (timestamp, packet) in packets {
                let findings = checks.process(player_state, packet, *timestamp);
                all_findings.extend(findings);
            }
        }
    }

    // Batch set states
    batch_set_states(&state, &server_id, &state_map)
        .await
        .map_err(map_bad_gateway("state batch-set failed"))?;

    // Post findings
    if !all_findings.is_empty() {
        post_findings(&state, &server_id, session_id.as_deref(), batch_id, all_findings, s3_key.as_deref())
            .await
            .map_err(map_bad_gateway("post findings failed"))?;
    }

    Ok(Json(serde_json::json!({
        "ok": true,
        "server_id": server_id,
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

async fn batch_get_states(
    state: &AppState,
    server_id: &str,
    player_uuids: &[Uuid],
) -> anyhow::Result<HashMap<Uuid, PlayerState>> {
    let url = format!("{}/callbacks/player-states/batch-get", state.api_base.trim_end_matches('/'));
    let req = BatchGetPlayerStatesRequest {
        server_id: server_id.to_string(),
        player_uuids: player_uuids.to_vec(),
        module_name: state.module_name.clone(),
    };

    let resp = state.http
        .post(&url)
        .header("authorization", format!("Bearer {}", state.module_callback_token))
        .json(&req)
        .send()
        .await
        .context("batch-get request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("batch-get http {}", resp.status());
    }

    let body: BatchGetPlayerStatesResponse = resp.json().await.context("batch-get parse failed")?;
    let mut out = HashMap::new();
    for row in body.states {
        let st: PlayerState = serde_json::from_value(row.state).unwrap_or_else(|_| PlayerState::new(row.player_uuid));
        out.insert(row.player_uuid, st);
    }
    Ok(out)
}

async fn batch_set_states(
    state: &AppState,
    server_id: &str,
    states: &HashMap<Uuid, PlayerState>,
) -> anyhow::Result<()> {
    let url = format!("{}/callbacks/player-states/batch-set", state.api_base.trim_end_matches('/'));
    let entries: Vec<PlayerStateEntry> = states
        .iter()
        .map(|(uuid, st)| PlayerStateEntry {
            player_uuid: *uuid,
            state: serde_json::to_value(st).unwrap_or_else(|_| serde_json::json!({})),
        })
        .collect();

    let req = BatchSetPlayerStatesRequest {
        server_id: server_id.to_string(),
        module_name: state.module_name.clone(),
        states: entries,
    };

    let resp = state.http
        .post(&url)
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
    s3_key: Option<&str>,
) -> anyhow::Result<()> {
    let url = format!("{}/callbacks/findings", state.api_base.trim_end_matches('/'));

    let finding_dtos: Vec<FindingDto> = findings
        .into_iter()
        .map(|f| {
            if let Some(key) = s3_key {
                let mut dto = FindingDto::from(f);
                dto.evidence_s3_key = Some(key.to_string());
                dto
            } else {
                FindingDto::from(f)
            }
        })
        .collect();

    let req = PostFindingsRequest {
        server_id: server_id.to_string(),
        session_id: session_id.map(|s| s.to_string()),
        batch_id,
        findings: finding_dtos,
    };

    let resp = state.http
        .post(&url)
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
