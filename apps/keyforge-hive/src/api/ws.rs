use crate::state::AppState;
use axum::{
    extract::{ws::Message, ws::WebSocket, State, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use keyforge_protocol::constants::{WS_MSG_CANCEL, WS_MSG_JOB};
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::interval;
use tracing::{debug, info, warn};

#[derive(Serialize)]
#[serde(tag = "type", content = "payload")]
enum ServerMessage {
    Job { id: String },
    Cancel { id: String },
}

pub async fn handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let rx = state.tx.subscribe();

    info!("🔌 Worker connected via WebSocket");

    // Task 1: Send Broadcasts & Heartbeats
    let send_task = tokio::spawn(async move {
        let mut rx = rx;
        // HIVE-050: Heartbeat Interval (30s)
        let mut heartbeat = interval(Duration::from_secs(30));

        loop {
            tokio::select! {
                // Broadcast Messages
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
                                if sender.send(Message::Text(json)).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(RecvError::Lagged(skipped)) => {
                            warn!("⚠️ WebSocket client lagged, skipped {} messages", skipped);
                        }
                        Err(RecvError::Closed) => break,
                    }
                }
                // Heartbeat Ping
                _ = heartbeat.tick() => {
                    if sender.send(Message::Ping(vec![])).await.is_err() {
                        break;
                    }
                    debug!("💓 Sent Ping");
                }
            }
        }
    });

    // Task 2: Receive Messages (Pong/Close) with Liveness Timeout
    // MINOR #58: Prevent memory leaks from hung connections
    loop {
        match tokio::time::timeout(Duration::from_secs(60), receiver.next()).await {
            Ok(Some(Ok(Message::Pong(_)))) => {
                debug!("💓 Received Pong");
            }
            Ok(Some(Ok(Message::Close(_)))) => break,
            Ok(None) => break, // Stream closed
            Err(_) => {
                warn!("🕒 WebSocket liveness timeout (no Pong for 60s)");
                break;
            }
            _ => {} // Ignore other messages
        }
    }

    send_task.abort();
    info!("�� Worker disconnected");
}
