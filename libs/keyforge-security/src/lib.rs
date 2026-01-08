// libs/keyforge-security/src/lib.rs

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


//! Security and cryptographic primitives for the KeyForge workspace.
//!
//! This crate provides wrappers for sensitive data (using zeroization),
//! utilities for Ed25519 digital signatures, and secure random nonce generation.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Errors that can occur during security operations.
#[derive(Error, Debug)]
pub enum SecurityError {
    /// Failed to encode or decode a cryptographic primitive (e.g., hex string parsing).
    #[error("Encoding Error: {0}")]
    Encoding(String),
    /// An issue with a secret or public key (e.g., invalid length or format).
    #[error("Key Error: {0}")]
    Key(String),
    /// Failed to create or verify a digital signature.
    #[error("Signature Error: {0}")]
    Signature(String),
}

/// A specialized Result type for security operations.
pub type SecurityResult<T> = Result<T, SecurityError>;

/// A wrapper for sensitive byte arrays that ensures data is zeroed out on drop.
///
/// Use this for storing raw keys or other sensitive binary data in memory to
/// mitigate the risk of data leakage after the value is no longer needed.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    /// Wraps a vector of bytes in a `SecretBytes` container.
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
    /// Returns a reference to the protected byte slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

/// A wrapper for sensitive strings that ensures data is zeroed out on drop.
///
/// Use this for storing passwords, API keys, or other sensitive text in memory.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretString(String);

impl SecretString {
    /// Wraps a string in a `SecretString` container.
    pub fn new(s: String) -> Self {
        Self(s)
    }
    /// Returns a reference to the protected string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Generates a new Ed25519 keypair and returns them as hex-encoded strings.
///
/// Returns a tuple of `(signing_key_hex, verifying_key_hex)`.
pub fn generate_keypair() -> (String, String) {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    (
        hex::encode(signing_key.to_bytes()),
        hex::encode(verifying_key.to_bytes()),
    )
}

fn build_payload(job_id: &str, layout: &str, score: f32, timestamp: u64, nonce: u64) -> Vec<u8> {
    let mut hasher = Sha256::new();

    hasher.update(job_id.as_bytes());
    let job_hash = hasher.finalize_reset();

    hasher.update(layout.as_bytes());
    let layout_hash = hasher.finalize_reset();

    let mut payload = Vec::with_capacity(32 + 32 + 4 + 8 + 8);
    payload.extend_from_slice(&job_hash);
    payload.extend_from_slice(&layout_hash);
    payload.extend_from_slice(&score.to_le_bytes());
    payload.extend_from_slice(&timestamp.to_le_bytes());
    payload.extend_from_slice(&nonce.to_le_bytes());

    payload
}

/// Signs an optimization result using a hex-encoded Ed25519 secret key.
///
/// The signature covers the `job_id`, `layout`, `score`, `timestamp`, and `nonce`.
/// Returns the hex-encoded signature string.
///
/// # Errors
/// Returns `SecurityError::Encoding` if the secret key is not a valid 64-character hex string.
pub fn sign_result(
    secret_hex: &str,
    job_id: &str,
    layout: &str,
    score: f32,
    timestamp: u64,
    nonce: u64,
) -> SecurityResult<String> {
    if secret_hex.len() != 64 {
        return Err(SecurityError::Encoding(
            "Key must be 64 hex characters".into(),
        ));
    }

    let secret_bytes = SecretBytes::new(
        hex::decode(secret_hex)
            .map_err(|_| SecurityError::Encoding("Invalid secret key hex".into()))?,
    );

    let signing_key = SigningKey::from_bytes(
        secret_bytes
            .as_slice()
            .try_into()
            .map_err(|_| SecurityError::Key("Invalid key length".into()))?,
    );

    sign_result_direct(&signing_key, job_id, layout, score, timestamp, nonce)
}

/// Signs an optimization result using a pre-loaded `SigningKey`.
///
/// This is the "direct" version of `sign_result` that avoids re-parsing the key.
/// The signature covers the `job_id`, `layout`, `score`, `timestamp`, and `nonce`.
/// Returns the hex-encoded signature string.
pub fn sign_result_direct(
    signing_key: &SigningKey,
    job_id: &str,
    layout: &str,
    score: f32,
    timestamp: u64,
    nonce: u64,
) -> SecurityResult<String> {
    let payload = build_payload(job_id, layout, score, timestamp, nonce);
    let signature = signing_key.sign(&payload);
    Ok(hex::encode(signature.to_bytes()))
}

/// Verifies a signed optimization result against a hex-encoded Ed25519 public key.
///
/// Rebuilds the payload from the provided parameters and checks it against the `signature_hex`.
/// Returns `Ok(true)` if the signature is valid, or `Ok(false)` if authentication fails.
///
/// # Errors
/// Returns `SecurityError` if any of the hex inputs are malformed or keys are invalid lengths.
pub fn verify_result(
    public_hex: &str,
    job_id: &str,
    layout: &str,
    score: f32,
    timestamp: u64,
    nonce: u64,
    signature_hex: &str,
) -> SecurityResult<bool> {
    if public_hex.len() != 64 {
        return Err(SecurityError::Encoding(
            "Key must be 64 hex characters".into(),
        ));
    }
    let public_bytes = hex::decode(public_hex)
        .map_err(|_| SecurityError::Encoding("Invalid public key hex".into()))?;

    let verifying_key = VerifyingKey::from_bytes(
        public_bytes
            .as_slice()
            .try_into()
            .map_err(|_| SecurityError::Key("Invalid key length".into()))?,
    )
    .map_err(|e| SecurityError::Key(e.to_string()))?;

    let signature_bytes = hex::decode(signature_hex)
        .map_err(|_| SecurityError::Encoding("Invalid signature hex".into()))?;

    let signature = Signature::from_bytes(
        signature_bytes
            .as_slice()
            .try_into()
            .map_err(|_| SecurityError::Signature("Invalid signature length".into()))?,
    );

    let payload = build_payload(job_id, layout, score, timestamp, nonce);

    match verifying_key.verify(&payload, &signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Generates a cryptographically secure random 64-bit nonce.
///
/// Used to prevent replay attacks in signed messages.
pub fn generate_nonce() -> u64 {
    use rand::RngCore;
    OsRng.next_u64()
}
