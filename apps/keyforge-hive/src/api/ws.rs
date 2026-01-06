// Copyright (c) 2025 KeyForge Contributors
//
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
    extract::{ws::Message, ws::WebSocket, State, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::interval;
use tracing::{debug, info, warn, error};
use keyforge_model::constants::{WS_MSG_JOB, WS_MSG_CANCEL};

/// Defines the external JSON protocol sent to WebSocket clients.
///
/// This enum is serialized with `#[serde(tag = "type", content = "payload")]`.
/// Example: `{"type": "Job", "payload": {"id": "123"}}`
#[derive(Serialize)]
#[serde(tag = "type", content = "payload")]
enum ServerMessage {
    /// Notifies workers that a new job is available in the queue.
    Job { id: String },
    /// Notifies workers to abort a specific job ID immediately.
    Cancel { id: String },
}

/// WebSocket upgrade handler.
///
/// Upgrades the HTTP connection to a WebSocket and spawns the connection handler.
pub async fn handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Manages the lifecycle of a single WebSocket connection.
///
/// Responsibilities:
/// 1. Subscribes to the internal broadcast channel (`state.tx`).
/// 2. Translates internal signals (e.g., "JOB:123") into external JSON messages.
/// 3. Sends periodic Heartbeat Pings to keep the connection alive.
/// 4. Monitors the connection for Pongs and Client Close frames.
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    
    // Subscribe to the internal broadcast bus.
    // This channel carries raw string signals from API handlers.
    let mut rx = state.tx.subscribe();
    info!("🔌 Worker connected via WebSocket. Subscribed to broadcast channel.");

    // Task 1: Outbound Loop (Broadcasts & Heartbeats)
    let send_task = tokio::spawn(async move {
        // Send a Ping every 30 seconds to detect dead connections.
        let mut heartbeat = interval(Duration::from_secs(30));

        loop {
            tokio::select! {
                // Handle Internal Broadcasts
                msg_res = rx.recv() => {
                    match msg_res {
                        Ok(msg_str) => {
                            // Parse the raw string signal using shared constants
                            let msg = if let Some(id) = msg_str.strip_prefix(WS_MSG_JOB) {
                                ServerMessage::Job { id: id.to_string() }
                            } else if let Some(id) = msg_str.strip_prefix(WS_MSG_CANCEL) {
                                ServerMessage::Cancel { id: id.to_string() }
                            } else {
                                warn!("⚠️ Unknown internal message format: '{}'", msg_str);
                                continue;
                            };

                            // Serialize to JSON and send to client
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

    // Task 2: Inbound Loop (Liveness Check)
    // We don't expect data from workers here (results go to POST /results),
    // but we must read the stream to process Pongs and Close frames.
    loop {
        // Enforce a 60s timeout. If the client doesn't respond to Pings (sent every 30s),
        // we assume the connection is dead and close it to free resources.
        match tokio::time::timeout(Duration::from_secs(60), receiver.next()).await {
            Ok(Some(Ok(Message::Pong(_)))) => {
                debug!("💓 Received Pong");
            }
            Ok(Some(Ok(Message::Close(_)))) => break,
            Ok(None) => break, // Stream closed by client
            Err(_) => {
                warn!("🕒 WebSocket liveness timeout (no Pong for 60s)");
                break;
            }
            _ => {} // Ignore other messages (Text/Binary)
        }
    }

    send_task.abort();
    info!(" Worker disconnected");
}