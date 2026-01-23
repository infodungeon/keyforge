// apps/keyforge-hive/src/features/register_node.rs

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

use crate::error::AppResult;
use crate::services::node_service::NodeService;
use crate::state::AppState;
use axum::{extract::State, Json};
use keyforge_model::Validator;
use keyforge_protocol::{NodeRequest, NodeResponse};
use std::sync::Arc;
use tracing::info;

/// VSA Feature: Register Node
/// Handles node heartbeat, identity verification, and auto-tuning calculations.
#[utoipa::path(
    post,
    path = "/nodes/register",
    request_body = NodeRequest,
    responses(
        (status = 200, description = "Node registered", body = NodeResponse)
    ),
    tag = "nodes"
)]
/// Handles a node registration or heartbeat request, performing identity verification and auto-tuning.
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NodeRequest>,
) -> AppResult<Json<NodeResponse>> {
    payload
        .validate()
        .map_err(crate::error::AppError::Validation)?;

    let node_id = payload.node_id.clone();
    let cpu_model = payload.cpu_model.clone();
    let ops = payload.ops_per_sec;

    let response = NodeService::register_node(&state, payload).await?;

    info!(
        "🖥️ Node Registered: {} | {} | {:.1} M/s",
        node_id,
        cpu_model,
        ops / 1_000_000.0
    );

    Ok(Json(response))
}
