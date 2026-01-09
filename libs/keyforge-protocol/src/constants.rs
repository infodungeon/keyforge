// libs/keyforge-protocol/src/constants.rs

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

//! Shared constants for the KeyForge Wire Protocol.

// Re-export domain constants for convenience
pub use keyforge_model::constants::{MAX_KEYBOARD_KEYS, MAX_PINNED_KEYS_COUNT};

// --- WebSocket Signaling ---

/// Prefix for internal Job broadcast messages.
/// Example: "JOB:12345"
pub const WS_MSG_JOB: &str = "JOB:";

/// Prefix for internal Cancel broadcast messages.
/// Example: "CANCEL:12345"
pub const WS_MSG_CANCEL: &str = "CANCEL:";

// --- Security Limits ---

/// Maximum number of biometric samples allowed in a single payload.
/// Note: Pending statistical research on optimal sample size.
pub const MAX_BIOMETRIC_SAMPLES: usize = 10_000;

// --- Temporal Policies ---

/// Maximum allowed future skew for result timestamps (seconds).
pub const MAX_FUTURE_SKEW_SEC: u64 = 300; // 5 minutes

/// Maximum allowed past age for result timestamps (seconds).
pub const MAX_PAST_SKEW_SEC: u64 = 1800; // 30 minutes
