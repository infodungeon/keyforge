// apps/keyforge-hive/src/api/ws.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::state::AppState;
use axum::{
    extract::{ws::Message, ws::WebSocket, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use keyforge_protocol::constants::{WS_MSG_CANCEL, WS_MSG_JOB};
use keyforge_protocol::NodeTelemetry;
use keyforge_security as crypto;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::constants::{DEFAULT_NODE_ID, WS_HEARTBEAT_INTERVAL_SECS, WS_LIVENESS_TIMEOUT_SECS};

/// Defines the external JSON protocol sent to WebSocket clients.
#[derive(Serialize)]
#[serde(tag = "type", content = "payload")]
enum ServerMessage {
    Job { id: String },
    Cancel { id: String },
}

/// Handles a WebSocket upgrade request, verifying the session token.
pub async fn handler(
    ws: WebSocketUpgrade,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let node_id = params
        .get("node_id")
        .cloned()
        .unwrap_or_else(|| DEFAULT_NODE_ID.to_string());

    // --- SECURE HANDSHAKE: Task-sec-022 ---
    // Extract and verify PASETO token from query params
    let token = params.get("token");
    let mut authenticated = false;

    if let Some(token_str) = token {
        let key = state.security.get_token_key();
        if let Ok(subject) = crypto::verify_paseto_token(&key, token_str) {
            if subject == node_id {
                authenticated = true;
            } else {
                warn!("⚠️ WS Auth: Token subject mismatch. Expected {}, got {}", node_id, subject);
            }
        }
    }

    if !authenticated {
        warn!("⛔ WS Auth Failed for node: {}", node_id);
        return StatusCode::UNAUTHORIZED.into_response();
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state, node_id))
}

/// Orchestrates the WebSocket lifecycle, managing outbound broadcasts and inbound telemetry.
async fn handle_socket(socket: WebSocket, state: Arc<AppState>, node_id: String) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to the internal broadcast bus (Process-Local)
    let mut rx = state.tx.subscribe();
    info!(" Worker connected via WebSocket: {}", node_id);

    // Task 1: Outbound Loop (Broadcasts & Heartbeats)
    let config_heartbeat = state
        .config
        .network
        .timeout_seconds
        .checked_div(2)
        .unwrap_or(WS_HEARTBEAT_INTERVAL_SECS);
    let send_task = tokio::spawn(async move {
        let mut heartbeat = interval(Duration::from_secs(config_heartbeat));

        loop {
            tokio::select! {
                // Handle Internal Broadcasts
                msg_res = rx.recv() => {
                    match msg_res {
                        Ok(msg_str) => {
                            let msg = if let Some(id) = msg_str.strip_prefix(WS_MSG_JOB) {
                                ServerMessage::Job { id: id.to_string() }
                            } else if let Some(id) = msg_str.strip_prefix(WS_MSG_CANCEL) {
                                ServerMessage::Cancel { id: id.to_string() }
                            } else {
                                continue;
                            };

                            if let Ok(json) = serde_json::to_string(&msg) {
                                if let Err(e) = sender.send(Message::Text(json.into())).await {
                                    error!("❌ Failed to send message to client: {}", e);
                                    break;
                                }
                            }
                        }
                        Err(RecvError::Lagged(skipped)) => {
                            warn!("⚠️ WebSocket client lagged, skipped {} messages", skipped);
                        }
                        Err(RecvError::Closed) => {
                            info!("Broadcast channel closed");
                            break;
                        }
                    }
                }
                // Handle Heartbeat Tick
                _ = heartbeat.tick() => {
                    if sender.send(Message::Ping(vec![].into())).await.is_err() {
                        break;
                    }
                    debug!("💓 Sent Ping");
                }
            }
        }
    });

    // Task 2: Inbound Loop (Telemetry & Liveness)
    let config_timeout = state
        .config
        .network
        .timeout_seconds
        .checked_mul(2)
        .unwrap_or(WS_LIVENESS_TIMEOUT_SECS);
    loop {
        // Enforce a timeout on inbound activity
        match tokio::time::timeout(Duration::from_secs(config_timeout), receiver.next()).await {
            Ok(Some(Ok(msg))) => match msg {
                Message::Pong(_) => {
                    debug!("💓 Received Pong");
                }
                Message::Text(text) => {
                    // Process Node Telemetry
                    if let Ok(telemetry) = serde_json::from_str::<NodeTelemetry>(&text) {
                        debug!(
                            "📊 Telemetry [{}]: IPS={:.1}, Temp={:.2}",
                            node_id, telemetry.ips, telemetry.temp
                        );

                        // Persist to Coordination Layer (Valkey)
                        if let Err(e) = state
                            .coordinator
                            .update_heartbeat(&node_id, &telemetry)
                            .await
                        {
                            warn!("Failed to update heartbeat for {}: {}", node_id, e);
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            },
            Ok(Some(Err(e))) => {
                warn!("WebSocket error: {}", e);
                break;
            }
            Ok(None) => break, // Stream closed
            Err(_) => {
                warn!("🕒 WebSocket liveness timeout (no activity for 60s)");
                break;
            }
        }
    }

    send_task.abort();
    info!("🔌 Worker disconnected: {}", node_id);
}
