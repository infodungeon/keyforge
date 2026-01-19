// apps/keyforge-agent/src/agent/crypto.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::agent::errors::{AgentError, AgentResult};
use keyforge_security as sec;

/// Signs an optimization result using the agent's private key.
///
/// This provides a stable interface for the network layer to sign results
/// before submitting them to the Hive.
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
        .map_err(|e| AgentError::Identity(format!("Signing error: {e}")))?;
    Ok(sig)
}
