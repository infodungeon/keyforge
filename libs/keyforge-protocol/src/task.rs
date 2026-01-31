// libs/keyforge-protocol/src/task.rs

use serde::{Deserialize, Serialize};
use crate::LimitedVec;
use crate::job::JobConfig;
use utoipa::ToSchema;

/// Represents a high-level research task that aggregates multiple jobs.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(ts_rs::TS), ts(export))]
pub struct ResearchTaskDto {
    /// Unique identifier for the research task.
    pub task_id: String,
    /// Name of the research campaign.
    pub name: String,
    /// List of job configurations to be executed.
    pub jobs: LimitedVec<JobConfig>,
    /// Current status of the research task.
    pub status: TaskStatusDto,
}

/// Represents the status of a research task.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[cfg_attr(feature = "ts_bindings", derive(ts_rs::TS), ts(export))]
pub enum TaskStatusDto {
    /// Task is waiting for workers.
    Pending,
    /// Task is currently being processed.
    Running {
        /// Number of jobs completed.
        completed: usize,
        /// Total number of jobs in the task.
        total: usize,
    },
    /// Task has finished successfully.
    Completed,
    /// Task failed to complete.
    Failed(String),
}

/// Status update from a worker node.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(ts_rs::TS), ts(export))]
pub struct TaskWorkerStatusDto {
    /// Unique ID of the worker.
    pub node_id: String,
    /// Current job ID being processed, if any.
    pub active_job_id: Option<String>,
    /// Worker capacity (e.g. core count).
    pub capacity: usize,
}
