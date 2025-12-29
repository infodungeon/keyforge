use keyforge_protocol::{
    config::{CorpusSource, LayoutDefinitions, ScoringWeights, SearchParams},
    geometry::{KeyNode, KeyboardDefinition, KeyboardGeometry, KeyboardMeta},
    JobConfig, JobQueueResponse, JobRequest, JobResponse, NodeRequest, NodeResponse,
    PopulationResponse, ResultSubmission, TuningProfile,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::features::register_job::handle,
        crate::features::get_queue::handle,
        crate::features::get_population::handle,
        crate::features::cancel_job::handle,
        crate::features::get_job_status::handle,
        crate::features::register_node::handle,
        crate::features::submit_result::handle,
        crate::features::submit_layout::handle,
        crate::features::list_submissions::handle,
        crate::features::nuke_user::handle,
        crate::features::system::health,
    ),
    components(
        schemas(
            JobRequest, JobResponse, JobQueueResponse, JobConfig,
            PopulationResponse, ResultSubmission,
            NodeRequest, NodeResponse, TuningProfile,
            KeyboardDefinition, KeyboardMeta, KeyboardGeometry, KeyNode,
            ScoringWeights, SearchParams, LayoutDefinitions, CorpusSource,
            crate::features::submit_layout::LayoutSubmission,
            crate::features::submit_layout::SubmissionResponse,
            crate::features::list_submissions::SubmissionEntry,
            crate::features::system::StatusResponse
        )
    ),
    tags(
        (name = "jobs", description = "Job Management"),
        (name = "nodes", description = "Compute Node Registration"),
        (name = "results", description = "Result Submission")
    )
)]
pub struct ApiDoc;
