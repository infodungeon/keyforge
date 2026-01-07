// apps/keyforge-agent/src/agent/crypto.rs

use crate::agent::errors::AgentResult;
use keyforge_security as sec;

pub fn sign_result_direct(
    private_key_hex: &str,
    job_id: &str,
    layout: &str,
    score: f32,
    timestamp: u64,
    nonce: u64,
) -> AgentResult<String> {
    // This wrapper bridges the Agent's error type to the Security crate
    let sig = sec::sign_result(private_key_hex, job_id, layout, score, timestamp, nonce)
        .map_err(|e| format!("Signing error: {}", e))?;
    Ok(sig)
}
