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
        crate::api::jobs::register_job,
        crate::api::jobs::get_queue,
        crate::api::jobs::get_population,
        crate::api::jobs::cancel_job,
        crate::api::nodes::register_node,
        crate::api::results::submit_result,
    ),
    components(
        schemas(
            JobRequest, JobResponse, JobQueueResponse, JobConfig,
            PopulationResponse, ResultSubmission,
            NodeRequest, NodeResponse, TuningProfile,
            KeyboardDefinition, KeyboardMeta, KeyboardGeometry, KeyNode,
            ScoringWeights, SearchParams, LayoutDefinitions, CorpusSource
        )
    ),
    tags(
        (name = "jobs", description = "Job Management"),
        (name = "nodes", description = "Compute Node Registration"),
        (name = "results", description = "Result Submission")
    )
)]
pub struct ApiDoc;
