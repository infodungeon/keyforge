use keyforge_protocol::{JobRequest, ResultSubmission};
// use keyforge_protocol::types::{JobId, NodeId}; // Assuming these exist or using String

#[allow(dead_code)]
#[derive(Debug)]
pub enum HiveCommand {
    RegisterJob(JobRequest),
    CancelJob(String),
    SubmitResult(ResultSubmission),
    RegisterNode {
        public_key: String, // Adjust based on NodeRegistration request
        // ...
    },
    // Add other commands as we migrate logic
}
