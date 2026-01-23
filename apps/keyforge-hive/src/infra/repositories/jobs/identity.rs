use keyforge_protocol::JobRequest;

pub fn calculate_job_identity(req: &JobRequest) -> Result<String, String> {
    req.config.id()
}
