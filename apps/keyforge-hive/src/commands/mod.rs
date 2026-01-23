// apps/keyforge-hive/src/commands/mod.rs

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

//! Reified business actions for the Hive service (Command Pattern).

use crate::error::AppResult;
use crate::services::job_service::JobService;
use crate::services::result_service::ResultService;
use crate::state::AppState;
use keyforge_protocol::{JobRequest, JobResponse, ResultSubmission};
use serde::{Deserialize, Serialize};

/// Reified Intent for Hive operations.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HiveCommand {
    /// Request to register a new optimization job.
    RegisterJob(Box<JobRequest>),
    /// Submit a result from a worker.
    SubmitResult(ResultSubmission),
    /// Cancel an active job.
    CancelJob { job_id: String },
}

/// Dispatches a command to the appropriate service.
/// Decouples Intent (Command) from Execution (Service).
pub async fn handle_command(state: &AppState, cmd: HiveCommand) -> AppResult<CommandResponse> {
    match cmd {
        HiveCommand::RegisterJob(req) => {
            let res = JobService::register_job(state, *req).await?;
            Ok(CommandResponse::JobRegistered(res))
        }
        HiveCommand::SubmitResult(res) => {
            let accepted = ResultService::submit_result(state, res).await?;
            Ok(CommandResponse::ResultAccepted { accepted })
        }
        HiveCommand::CancelJob { .. } => todo!("Implement remaining command handlers"),
    }
}

/// Unified response type for commands.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CommandResponse {
    /// Result of `RegisterJob`.
    JobRegistered(JobResponse),
    /// Result of `SubmitResult`.
    ResultAccepted { accepted: bool },
    /// Result of `CancelJob`.
    JobCancelled { job_id: String },
}
