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
use std::{collections::HashMap, io::Read};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    api_base: String,
    module_callback_token: String,
    speed_threshold_bps: f64,
    http: reqwest::Client,
}

#[derive(Deserialize)]
struct BatchMeta {
    server_id: Option<String>,
    session_id: Option<String>,
    created_at_ms: Option<u64>,
    event_count: Option<usize>,
}

#[derive(Deserialize)]
struct EventLine {
    ts: u64,
    dir: Option<String>,
    pkt: String,
    uuid: Option<String>,
    name: Option<String>,
    fields: Option<Value>,
    // For transformed streams (e.g. movement_events_v1), coordinates may be top-level.
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
}

#[derive(Serialize)]
struct FindingOut {
    player_uuid: Option<Uuid>,
    detector_name: String,
    detector_version: Option<String>,
    severity: String,
    title: String,
    description: Option<String>,
    evidence_s3_key: Option<String>,
    evidence_json: Option<Value>,
}

#[derive(Serialize)]
struct PostFindingsRequest {
    server_id: String,
    session_id: Option<String>,
    batch_id: Option<Uuid>,
    findings: Vec<FindingOut>,
}

#[derive(Serialize)]
struct Health {
    ok: bool,
    name: &'static str,
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
        .unwrap_or(4010);

    let api_base = std::env::var("API_BASE").unwrap_or_else(|_| "http://127.0.0.1:3002".to_string());
    let module_callback_token = std::env::var("MODULE_CALLBACK_TOKEN").unwrap_or_default();
    let speed_threshold_bps = std::env::var("SPEED_THRESHOLD_BPS")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(50.0);

    if module_callback_token.is_empty() {
        tracing::warn!("MODULE_CALLBACK_TOKEN is empty; callbacks will be rejected by the API.");
    }

    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let state = AppState {
        api_base,
        module_callback_token,
        speed_threshold_bps,
        http,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/ingest", post(ingest))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", host, port).parse()?;
    tracing::info!("demo_module listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

async fn health() -> Json<Health> {
    Json(Health {
        ok: true,
        name: "demo_module",
    })
}

async fn ingest(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, (axum::http::StatusCode, Json<Value>)> {
    // --- Extract headers ---
    let server_id = headers
        .get("x-server-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
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

    if server_id.is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error":"missing x-server-id"})),
        ));
    }

    // --- Decompress ---
    let mut decoder = GzDecoder::new(body.as_ref());
    let mut text = String::new();
    decoder
        .read_to_string(&mut text)
        .map_err(|e| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("gzip decode failed: {}", e)})),
            )
        })?;

    // --- Parse NDJSON ---
    let mut lines = text.lines();
    let meta_line = lines.next().unwrap_or("{}");
    let _meta: BatchMeta = serde_json::from_str(meta_line).unwrap_or(BatchMeta {
        server_id: None,
        session_id: None,
        created_at_ms: None,
        event_count: None,
    });

    // Per-player last position
    #[derive(Clone, Copy)]
    struct Pos {
        ts: u64,
        x: f64,
        y: f64,
        z: f64,
    }

    let mut last: HashMap<Uuid, Pos> = HashMap::new();
    let mut findings: Vec<FindingOut> = Vec::new();

    for line in lines {
        let ev: EventLine = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let Some(uuid_str) = ev.uuid.as_deref() else { continue };
        let Ok(player_uuid) = Uuid::parse_str(uuid_str) else { continue };

        // Coordinates can be either in `fields` (raw packets) or top-level (transforms).
        let mut x = ev.x;
        let mut y = ev.y;
        let mut z = ev.z;
        if x.is_none() || y.is_none() || z.is_none() {
            if let Some(fields) = ev.fields.as_ref().and_then(|v| v.as_object()) {
                x = x.or_else(|| fields.get("x").and_then(|v| v.as_f64()));
                y = y.or_else(|| fields.get("y").and_then(|v| v.as_f64()));
                z = z.or_else(|| fields.get("z").and_then(|v| v.as_f64()));
            }
        }
        let (Some(x), Some(y), Some(z)) = (x, y, z) else { continue };

        if let Some(prev) = last.get(&player_uuid).copied() {
            if ev.ts < prev.ts {
                findings.push(FindingOut {
                    player_uuid: Some(player_uuid),
                    detector_name: "demo_time_monotonic".to_string(),
                    detector_version: Some("1".to_string()),
                    severity: "warning".to_string(),
                    title: "Timestamp went backwards".to_string(),
                    description: Some(format!("ts={} prev_ts={}", ev.ts, prev.ts)),
                    evidence_s3_key: s3_key.clone(),
                    evidence_json: Some(serde_json::json!({
                        "pkt": ev.pkt,
                        "ts": ev.ts,
                        "prev_ts": prev.ts
                    })),
                });
                // Do not overwrite last-known position with out-of-order data.
                continue;
            } else {
                let dt = (ev.ts - prev.ts) as f64 / 1000.0;
                if dt > 0.0 {
                    let dx = x - prev.x;
                    let dy = y - prev.y;
                    let dz = z - prev.z;
                    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                    let bps = dist / dt;
                    if bps > state.speed_threshold_bps {
                        findings.push(FindingOut {
                            player_uuid: Some(player_uuid),
                            detector_name: "demo_speed_check".to_string(),
                            detector_version: Some("1".to_string()),
                            severity: "warning".to_string(),
                            title: "Unusually high movement speed".to_string(),
                            description: Some(format!("speed_bps={:.2} threshold={:.2}", bps, state.speed_threshold_bps)),
                            evidence_s3_key: s3_key.clone(),
                            evidence_json: Some(serde_json::json!({
                                "pkt": ev.pkt,
                                "from": {"ts": prev.ts, "x": prev.x, "y": prev.y, "z": prev.z},
                                "to": {"ts": ev.ts, "x": x, "y": y, "z": z},
                                "bps": bps,
                                "threshold_bps": state.speed_threshold_bps
                            })),
                        });
                    }
                }
            }
        }

        last.insert(player_uuid, Pos { ts: ev.ts, x, y, z });
    }

    // Always emit a proof-of-wiring finding if none were produced
    if findings.is_empty() {
        findings.push(FindingOut {
            player_uuid: None,
            detector_name: "demo_module".to_string(),
            detector_version: Some("1".to_string()),
            severity: "info".to_string(),
            title: "Demo module processed batch".to_string(),
            description: Some("No violations detected; this is a wiring check.".to_string()),
            evidence_s3_key: s3_key.clone(),
            evidence_json: Some(serde_json::json!({"batch_id": batch_id})),
        });
    }

    // --- Callback ---
    let url = format!("{}/callbacks/findings", state.api_base.trim_end_matches('/'));
    let req_body = PostFindingsRequest {
        server_id: server_id.clone(),
        session_id: session_id.clone(),
        batch_id,
        findings,
    };

    let resp = state
        .http
        .post(url)
        .header("authorization", format!("Bearer {}", state.module_callback_token))
        .json(&req_body)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => Ok(Json(serde_json::json!({"ok": true}))),
        Ok(r) => Err((
            axum::http::StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": format!("callback failed: http {}", r.status())})),
        )),
        Err(e) => Err((
            axum::http::StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": format!("callback error: {}", e)})),
        )),
    }
}


