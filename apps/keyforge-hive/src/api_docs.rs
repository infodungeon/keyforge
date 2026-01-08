// apps/keyforge-hive/src/api_docs.rs

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


use keyforge_model::{
    CorpusSource, KeyNode, KeyboardDefinition, KeyboardGeometry, ScoringWeights, SearchParams,
};
use keyforge_protocol::{
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
            KeyboardDefinition, KeyboardGeometry, KeyNode,
            ScoringWeights, SearchParams, CorpusSource,
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
