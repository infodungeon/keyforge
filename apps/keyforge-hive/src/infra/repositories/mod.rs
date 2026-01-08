// apps/keyforge-hive/src/infra/repositories/mod.rs

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

/// Security audit log persistence.
pub mod audit;
/// Optimization job lifecycle management.
pub mod jobs;
/// Worker node registry and telemetry management.
pub mod nodes;
/// Optimization result storage.
pub mod results;
/// User layout submission persistence.
pub mod submissions;
/// User account and profile management.
pub mod users;

pub use audit::AuditRepository;
pub use jobs::JobRepository;
pub use nodes::NodeRepository;
pub use results::ResultRepository;
pub use submissions::SubmissionRepository;
pub use users::UserRepository;
