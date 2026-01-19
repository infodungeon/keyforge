// apps/keyforge-hive/src/features/mod.rs

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

/// Endpoint for cancelling an active job.
pub mod cancel_job;
/// Endpoint for checking the status of a specific job.
pub mod get_job_status;
/// Endpoint for retrieving the top population for a specific job.
pub mod get_population;
/// Endpoint for retrieving the current job queue.
pub mod get_queue;
/// Endpoint for listing all user-submitted layouts.
pub mod list_submissions;
/// Administrative endpoint for deleting all user data (GDPR/Test cleanup).
pub mod nuke_user;
/// Endpoint for registering new optimization jobs.
pub mod register_job;
/// Endpoint for registering a new worker node.
pub mod register_node;
/// Endpoint for submitting a custom layout for persistent storage.
pub mod submit_layout;
/// Endpoint for workers to submit optimization results.
pub mod submit_result;
/// General system status and version info.
pub mod system;
