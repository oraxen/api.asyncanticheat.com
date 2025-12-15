//! Vulcan Module - Main entry point
//!
//! Vulcan-style anti-cheat module for AsyncAnticheat

use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

mod buffer;
mod checks;
mod config;
mod findings;
mod packets;
mod player_state;

use checks::{CombatChecks, MovementChecks, PlayerChecks};
use config::VulcanConfig;
use findings::Finding;
use packets::PacketRecord;
use player_state::PlayerState;

/// Module configuration
#[derive(Debug, Clone)]
struct ModuleConfig {
    api_url: String,
    callback_token: String,
    host: String,
    port: u16,
    player_timeout_ms: i64,
}

impl ModuleConfig {
    fn from_env() -> Self {
        Self {
            api_url: std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:3002".to_string()),
            callback_token: std::env::var("CALLBACK_TOKEN").unwrap_or_else(|_| "dev".to_string()),
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(4012),
            player_timeout_ms: std::env::var("PLAYER_TIMEOUT_MS")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(300_000),
        }
    }
}

/// Application state
struct AppState {
    vulcan_config: RwLock<VulcanConfig>,
    module_config: ModuleConfig,
    player_states: DashMap<Uuid, PlayerState>,
    combat_checks: CombatChecks,
    movement_checks: MovementChecks,
    player_checks: PlayerChecks,
    http_client: reqwest::Client,
}

impl AppState {
    fn new(module_config: ModuleConfig) -> Self {
        let vulcan_config = VulcanConfig::default();
        
        Self {
            combat_checks: CombatChecks::new(vulcan_config.clone()),
            movement_checks: MovementChecks::new(vulcan_config.clone()),
            player_checks: PlayerChecks::new(vulcan_config.clone()),
            vulcan_config: RwLock::new(vulcan_config),
            module_config,
            player_states: DashMap::new(),
            http_client: reqwest::Client::builder()
                // Callbacks may involve DB writes; keep this comfortably above the API's
                // typical p99 so we don't drop findings under load.
                .timeout(Duration::from_secs(20))
                .build()
                .unwrap(),
        }
    }

    async fn get_or_create_player(
        &self,
        player_uuid: Uuid,
        player_name: String,
        timestamp_ms: i64,
    ) -> dashmap::mapref::one::RefMut<'_, Uuid, PlayerState> {
        if !self.player_states.contains_key(&player_uuid) {
            let config = self.vulcan_config.read().await;
            let state = PlayerState::new(player_uuid, player_name, &config, timestamp_ms);
            drop(config);
            self.player_states.insert(player_uuid, state);
        }
        self.player_states.get_mut(&player_uuid).unwrap()
    }
}

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    ok: bool,
    module: &'static str,
    version: &'static str,
}

/// Batch processing request
#[derive(Deserialize)]
struct ProcessBatchRequest {
    server_id: String,
    session_id: String,
    batch_id: String,
    packets: Vec<PacketRecord>,
}

/// Batch processing response
#[derive(Serialize)]
struct ProcessBatchResponse {
    ok: bool,
    findings_count: usize,
    packets_processed: usize,
}

/// Finding callback request
#[derive(Serialize)]
struct FindingOut {
    player_uuid: Option<Uuid>,
    detector_name: String,
    detector_version: Option<String>,
    severity: Option<String>,
    title: String,
    description: Option<String>,
    evidence_s3_key: Option<String>,
    evidence_json: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct PostFindingsRequest {
    server_id: String,
    session_id: Option<String>,
    batch_id: Option<Uuid>,
    findings: Vec<FindingOut>,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        module: "vulcan_module",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn process_batch(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ProcessBatchRequest>,
) -> Result<Json<ProcessBatchResponse>, StatusCode> {
    let start = std::time::Instant::now();
    let mut all_findings: Vec<(Finding, String)> = Vec::new();
    let packets_count = request.packets.len();

    debug!(
        "Processing batch {} with {} packets for server {}",
        request.batch_id, packets_count, request.server_id
    );

    for packet_record in &request.packets {
        let parsed = packet_record.parse();
        let player_uuid = packet_record.player_uuid;
        let player_name = packet_record
            .player_name
            .clone()
            .unwrap_or_else(|| player_uuid.to_string());
        let timestamp_ms = packet_record.timestamp_ms;

        let mut player_state = state
            .get_or_create_player(player_uuid, player_name.clone(), timestamp_ms)
            .await;
        player_state.touch(timestamp_ms);

        // Run all check categories
        let mut findings = Vec::new();
        findings.extend(state.combat_checks.process(&mut player_state, &parsed, timestamp_ms));
        findings.extend(state.movement_checks.process(&mut player_state, &parsed, timestamp_ms));
        findings.extend(state.player_checks.process(&mut player_state, &parsed, timestamp_ms));

        for finding in findings {
            all_findings.push((finding, player_name.clone()));
        }
    }

    let findings_count = all_findings.len();

    // Send findings to API
    if !all_findings.is_empty() {
        let base = state.module_config.api_url.trim_end_matches('/');
        let api_url = format!("{}/callbacks/findings", base);
        let http_client = state.http_client.clone();
        let token = state.module_config.callback_token.clone();
        let server_id = request.server_id.clone();
        let session_id = Some(request.session_id.clone());
        let batch_uuid = request.batch_id.parse::<Uuid>().ok();
        let batch_id_str = request.batch_id.clone();

        let findings: Vec<FindingOut> = all_findings
            .into_iter()
            .map(|(finding, _player_name)| FindingOut {
                player_uuid: Some(finding.player_uuid),
                detector_name: finding.feature.detector_name().to_string(),
                detector_version: Some(env!("CARGO_PKG_VERSION").to_string()),
                severity: Some(finding.severity.as_str().to_string()),
                title: finding.title.clone(),
                description: finding.description.clone(),
                evidence_s3_key: None,
                evidence_json: finding.evidence.clone(),
            })
            .collect();

        let payload = PostFindingsRequest {
            server_id,
            session_id,
            batch_id: batch_uuid,
            findings,
        };

        tokio::spawn(async move {
            let findings_len = payload.findings.len();
            let mut attempt: u32 = 0;
            loop {
                attempt += 1;
                let res = http_client
                    .post(&api_url)
                    .header("Authorization", format!("Bearer {}", token))
                    .json(&payload)
                    .send()
                    .await;

                match res {
                    Ok(resp) if resp.status().is_success() => {
                        debug!(
                            "Findings callback ok batch_id={} findings={} attempt={}",
                            batch_id_str, findings_len, attempt
                        );
                        break;
                    }
                    Ok(resp) => {
                        let status = resp.status();
                        let body = resp.text().await.unwrap_or_default();
                        warn!(
                            "Findings callback failed batch_id={} findings={} attempt={} status={} body={}",
                            batch_id_str, findings_len, attempt, status, body
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Findings callback error batch_id={} findings={} attempt={} err={:?}",
                            batch_id_str, findings_len, attempt, e
                        );
                    }
                }

                if attempt >= 5 {
                    error!(
                        "Findings callback giving up batch_id={} findings={} after {} attempts",
                        batch_id_str, findings_len, attempt
                    );
                    break;
                }

                // Exponential backoff (200ms, 400ms, 800ms, 1600ms, 3200ms)
                let backoff_ms = 200u64.saturating_mul(1u64 << (attempt - 1));
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }
        });
    }

    let elapsed = start.elapsed();
    debug!(
        "Batch {} processed in {:?}: {} packets, {} findings",
        request.batch_id, elapsed, packets_count, findings_count
    );

    Ok(Json(ProcessBatchResponse {
        ok: true,
        findings_count,
        packets_processed: packets_count,
    }))
}

async fn get_config(State(state): State<Arc<AppState>>) -> Json<VulcanConfig> {
    let config = state.vulcan_config.read().await;
    Json(config.clone())
}

async fn update_config(
    State(state): State<Arc<AppState>>,
    Json(new_config): Json<VulcanConfig>,
) -> Json<VulcanConfig> {
    let mut config = state.vulcan_config.write().await;
    *config = new_config;
    Json(config.clone())
}

#[derive(Serialize)]
struct PlayerStatesResponse {
    count: usize,
    players: Vec<serde_json::Value>,
}

async fn get_player_states(State(state): State<Arc<AppState>>) -> Json<PlayerStatesResponse> {
    let players: Vec<serde_json::Value> = state
        .player_states
        .iter()
        .map(|entry| entry.value().to_json())
        .collect();

    Json(PlayerStatesResponse {
        count: players.len(),
        players,
    })
}

async fn cleanup_states(state: Arc<AppState>) {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        
        let now = chrono::Utc::now().timestamp_millis();
        let timeout = state.module_config.player_timeout_ms;

        let stale_players: Vec<Uuid> = state
            .player_states
            .iter()
            .filter(|entry| entry.value().is_stale(now, timeout))
            .map(|entry| *entry.key())
            .collect();

        if !stale_players.is_empty() {
            info!("Cleaning up {} stale player states", stale_players.len());
            for uuid in stale_players {
                state.player_states.remove(&uuid);
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("vulcan_module=debug".parse()?)
                .add_directive("tower_http=debug".parse()?),
        )
        .init();

    let module_config = ModuleConfig::from_env();
    let bind_addr = format!("{}:{}", module_config.host, module_config.port);

    info!("Starting Vulcan Module v{}", env!("CARGO_PKG_VERSION"));
    info!("API URL: {}", module_config.api_url);
    info!("Listening on: {}", bind_addr);

    let state = Arc::new(AppState::new(module_config));

    let cleanup_state = state.clone();
    tokio::spawn(cleanup_states(cleanup_state));

    let app = Router::new()
        .route("/health", get(health))
        .route("/process", post(process_batch))
        .route("/config", get(get_config))
        .route("/config", post(update_config))
        .route("/players", get(get_player_states))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

