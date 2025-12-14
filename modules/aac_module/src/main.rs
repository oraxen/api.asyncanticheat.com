//! AAC Module - Main entry point
//!
//! This module connects to the AsyncAnticheat API and processes packet batches
//! using AAC5-style checks.

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

mod checks;
mod config;
mod findings;
mod packets;
mod player_state;
mod vl;

use checks::{
    AimbotCheck, AutoclickerCheck, DelaysCheck, HitboxCheck, InteractCheck, MiscCheck,
    MovementCheck,
};
use config::AacConfig;
use findings::Finding;
use packets::PacketRecord;
use player_state::PlayerState;

/// Module configuration
#[derive(Debug, Clone)]
struct ModuleConfig {
    /// API base URL for callbacks
    api_url: String,
    /// Callback token for authentication
    callback_token: String,
    /// Server to listen on
    host: String,
    /// Port to listen on
    port: u16,
    /// Player state timeout (ms)
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
                .unwrap_or(4011),
            player_timeout_ms: std::env::var("PLAYER_TIMEOUT_MS")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(300_000), // 5 minutes
        }
    }
}

/// Application state
struct AppState {
    /// AAC configuration (shared, can be updated at runtime)
    aac_config: RwLock<AacConfig>,
    /// Module configuration
    module_config: ModuleConfig,
    /// Per-player state
    player_states: DashMap<Uuid, PlayerState>,
    /// HTTP client for callbacks
    http_client: reqwest::Client,
}

impl AppState {
    fn new(module_config: ModuleConfig) -> Self {
        let aac_config = AacConfig::default();
        
        Self {
            aac_config: RwLock::new(aac_config),
            module_config,
            player_states: DashMap::new(),
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
        }
    }

    /// Get fresh check instances using the current config
    /// This ensures config updates take effect immediately
    async fn get_checks(&self) -> (DelaysCheck, MovementCheck, AimbotCheck, AutoclickerCheck, HitboxCheck, InteractCheck, MiscCheck) {
        let config = self.aac_config.read().await;
        (
            DelaysCheck::new(config.delays.clone()),
            MovementCheck::new(config.r#move.clone()),
            AimbotCheck::new(config.aimbot.clone()),
            AutoclickerCheck::new(config.autoclicker.clone()),
            HitboxCheck::new(config.hitbox.clone()),
            InteractCheck::new(config.interact.clone()),
            MiscCheck::new(config.misc.clone()),
        )
    }

    fn get_or_create_player(&self, player_uuid: Uuid, player_name: String, timestamp_ms: i64) -> dashmap::mapref::one::RefMut<'_, Uuid, PlayerState> {
        // Use entry API for atomic get-or-insert to prevent race conditions
        // This avoids the contains_key + insert pattern which can cause overwrites
        self.player_states.entry(player_uuid).or_insert_with(|| {
            // Note: We can't easily hold aac_config lock here due to async,
            // but the default config is acceptable for initial state creation
            let config = AacConfig::default();
            PlayerState::new(player_uuid, player_name.clone(), &config, timestamp_ms)
        })
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

/// Finding callback request (sent to API)
#[derive(Serialize)]
struct FindingCallbackRequest {
    server_id: String,
    player_uuid: String,
    player_name: Option<String>,
    detector_name: String,
    severity: String,
    title: String,
    description: Option<String>,
    evidence: Option<serde_json::Value>,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        module: "aac_module",
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

    // Get fresh checks with current config (ensures config updates take effect)
    let (delays_check, movement_check, aimbot_check, autoclicker_check, hitbox_check, interact_check, misc_check) = 
        state.get_checks().await;

    for packet_record in &request.packets {
        let parsed = packet_record.parse();
        let player_uuid = packet_record.player_uuid;
        let player_name = packet_record
            .player_name
            .clone()
            .unwrap_or_else(|| player_uuid.to_string());
        let timestamp_ms = packet_record.timestamp_ms;

        // Get or create player state
        let mut player_state = state.get_or_create_player(player_uuid, player_name.clone(), timestamp_ms);
        player_state.touch(timestamp_ms);

        // Run all enabled checks
        let mut findings = Vec::new();

        findings.extend(delays_check.process(&mut player_state, &parsed, timestamp_ms));
        findings.extend(movement_check.process(&mut player_state, &parsed, timestamp_ms));
        findings.extend(aimbot_check.process(&mut player_state, &parsed, timestamp_ms));
        findings.extend(autoclicker_check.process(&mut player_state, &parsed, timestamp_ms));
        findings.extend(hitbox_check.process(&mut player_state, &parsed, timestamp_ms));
        findings.extend(interact_check.process(&mut player_state, &parsed, timestamp_ms));
        findings.extend(misc_check.process(&mut player_state, &parsed, timestamp_ms));

        for finding in findings {
            all_findings.push((finding, player_name.clone()));
        }
    }

    let findings_count = all_findings.len();

    // Send findings to API
    if !all_findings.is_empty() {
        let api_url = format!("{}/module/findings", state.module_config.api_url);
        
        for (finding, player_name) in all_findings {
            let callback_req = FindingCallbackRequest {
                server_id: request.server_id.clone(),
                player_uuid: finding.player_uuid.to_string(),
                player_name: Some(player_name),
                detector_name: finding.feature.detector_name().to_string(),
                severity: finding.severity.as_str().to_string(),
                title: finding.title.clone(),
                description: finding.description.clone(),
                evidence: finding.evidence.clone(),
            };

            let http_client = state.http_client.clone();
            let token = state.module_config.callback_token.clone();
            let url = api_url.clone();

            // Send callback asynchronously
            tokio::spawn(async move {
                match http_client
                    .post(&url)
                    .header("Authorization", format!("Bearer {}", token))
                    .json(&callback_req)
                    .send()
                    .await
                {
                    Ok(resp) if resp.status().is_success() => {
                        debug!("Finding callback sent successfully");
                    }
                    Ok(resp) => {
                        warn!("Finding callback failed with status: {}", resp.status());
                    }
                    Err(e) => {
                        error!("Finding callback error: {}", e);
                    }
                }
            });
        }
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

/// Get current configuration
async fn get_config(State(state): State<Arc<AppState>>) -> Json<AacConfig> {
    let config = state.aac_config.read().await;
    Json(config.clone())
}

/// Update configuration
async fn update_config(
    State(state): State<Arc<AppState>>,
    Json(new_config): Json<AacConfig>,
) -> Json<AacConfig> {
    let mut config = state.aac_config.write().await;
    *config = new_config;
    Json(config.clone())
}

/// Get player states (for debugging)
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

/// Cleanup stale player states
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
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("aac_module=debug".parse()?)
                .add_directive("tower_http=debug".parse()?),
        )
        .init();

    let module_config = ModuleConfig::from_env();
    let bind_addr = format!("{}:{}", module_config.host, module_config.port);

    info!("Starting AAC Module v{}", env!("CARGO_PKG_VERSION"));
    info!("API URL: {}", module_config.api_url);
    info!("Listening on: {}", bind_addr);

    let state = Arc::new(AppState::new(module_config));

    // Spawn cleanup task
    let cleanup_state = state.clone();
    tokio::spawn(cleanup_states(cleanup_state));

    // Build router
    let app = Router::new()
        .route("/health", get(health))
        .route("/process", post(process_batch))
        .route("/config", get(get_config))
        .route("/config", post(update_config))
        .route("/players", get(get_player_states))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    // Start server
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

