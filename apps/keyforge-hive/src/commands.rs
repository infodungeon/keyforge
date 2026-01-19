// apps/keyforge-hive/src/commands.rs

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

use keyforge_protocol::{JobRequest, ResultSubmission};
// use keyforge_protocol::types::{JobId, NodeId}; // Assuming these exist or using String

#[allow(dead_code)]
#[derive(Debug)]
pub enum HiveCommand {
    RegisterJob(Box<JobRequest>),
    CancelJob(String),
    SubmitResult(ResultSubmission),
    RegisterNode {
        public_key: String, // Adjust based on NodeRegistration request
        // ...
    },
    // Add other commands as we migrate logic
}
