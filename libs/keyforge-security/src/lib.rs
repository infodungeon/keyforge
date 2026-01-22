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

//! Security and cryptographic primitives for the `KeyForge` workspace.
//!
//! This crate provides wrappers for sensitive data (using zeroization),
//! utilities for Ed25519 digital signatures, and secure random nonce generation.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use keyforge_model::constants::SCORE_SCALE;
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

impl std::fmt::Debug for SecretBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SecretBytes")
            .field(&"***REDACTED***")
            .finish()
    }
}

impl SecretBytes {
    /// Wraps a vector of bytes in a `SecretBytes` container.
    #[must_use]
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
    /// Returns a reference to the protected byte slice.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

/// A wrapper for sensitive strings that ensures data is zeroed out on drop.
///
/// Use this for storing passwords, API keys, or other sensitive text in memory.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretString(String);

impl std::fmt::Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SecretString")
            .field(&"***REDACTED***")
            .finish()
    }
}

impl SecretString {
    /// Wraps a string in a `SecretString` container.
    #[must_use]
    pub fn new(s: String) -> Self {
        Self(s)
    }
    /// Returns a reference to the protected string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Generates a new Ed25519 keypair and returns them as hex-encoded strings.
///
/// Returns a tuple of `(signing_key_hex, verifying_key_hex)`.
#[must_use]
pub fn generate_keypair() -> (String, String) {
    let mut bytes = [0u8; 32];
    let mut csprng = rand::rng();
    rand::RngCore::fill_bytes(&mut csprng, &mut bytes);
    let signing_key = SigningKey::from_bytes(&bytes);
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

    // Task-sec-029: Use fixed-point representation for deterministic signatures
    #[allow(clippy::cast_possible_truncation)]
    let score_fixed = (score * SCORE_SCALE) as i64;

    // 32 bytes (job_hash) + 32 bytes (layout_hash) + 8 bytes (score) + 8 bytes (timestamp) + 8 bytes (nonce)
    let capacity = 32 + 32 + 8 + 8 + 8;
    let mut payload = Vec::with_capacity(capacity);
    payload.extend_from_slice(&job_hash);
    payload.extend_from_slice(&layout_hash);
    payload.extend_from_slice(&score_fixed.to_le_bytes());
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
    let secret_hex = secret_hex.trim();

    // Task-sec-027: Use direct decoding into a zeroized buffer to avoid leakage
    let mut key_buf = [0u8; 32];
    hex::decode_to_slice(secret_hex, &mut key_buf)
        .map_err(|_| SecurityError::Encoding("Invalid secret key hex".into()))?;

    let signing_key = SigningKey::from_bytes(&key_buf);
    key_buf.zeroize();

    sign_result_direct(&signing_key, job_id, layout, score, timestamp, nonce)
}

/// Signs an optimization result using a pre-loaded `SigningKey`.
///
/// This is the "direct" version of `sign_result` that avoids re-parsing the key.
/// The signature covers the `job_id`, `layout`, `score`, `timestamp`, and `nonce`.
/// Returns the hex-encoded signature string.
///
/// # Errors
/// This function is currently infallible but returns a `Result` for consistency with the
/// `sign_result` API and to support future cryptographic backends that may fail.
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
    let public_hex = public_hex.trim();
    let signature_hex = signature_hex.trim();

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
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Generates a cryptographically secure random 64-bit nonce.
///
/// Used to prevent replay attacks in signed messages.
#[must_use]
pub fn generate_nonce() -> u64 {
    use rand::Rng;
    rand::rng().random()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify_happy_path() {
        let (secret, public) = generate_keypair();
        let job_id = "job-123";
        let layout = "qwerty...";
        let score = 98.6;
        let timestamp = 1_234_567_890;
        let nonce = generate_nonce();

        let sig = sign_result(&secret, job_id, layout, score, timestamp, nonce).unwrap();
        let valid = verify_result(&public, job_id, layout, score, timestamp, nonce, &sig).unwrap();

        assert!(valid, "Signature should verify correctly");
    }

    #[test]
    fn test_sign_with_whitespace() {
        let (secret, public) = generate_keypair();
        // Add whitespace to secret
        let spaced_secret = format!("  {secret}  ");

        let sig = sign_result(&spaced_secret, "job", "layout", 1.0, 0, 0).unwrap();
        let valid = verify_result(&public, "job", "layout", 1.0, 0, 0, &sig).unwrap();
        assert!(valid, "Detailed whitespace should be trimmed");
    }

    #[test]
    fn test_verify_with_whitespace() {
        let (secret, public) = generate_keypair();
        let sig = sign_result(&secret, "job", "layout", 1.0, 0, 0).unwrap();

        let spaced_public = format!("\n{public}\t");
        let spaced_sig = format!(" {sig} ");

        let valid = verify_result(&spaced_public, "job", "layout", 1.0, 0, 0, &spaced_sig).unwrap();
        assert!(
            valid,
            "Public key and signature whitespace should be trimmed"
        );
    }

    #[test]
    fn test_invalid_hex() {
        let res = sign_result("not-hex-at-all", "job", "layout", 1.0, 0, 0);
        assert!(matches!(res, Err(SecurityError::Encoding(_))));
    }

    #[test]
    fn test_verify_tampered_payload() {
        let (secret, public) = generate_keypair();
        let sig = sign_result(&secret, "job", "layout", 1.0, 0, 0).unwrap();

        // Check with different score
        let valid = verify_result(&public, "job", "layout", 99.0, 0, 0, &sig).unwrap();
        assert!(!valid, "Tampered payload should verify as false");
    }

    #[test]
    fn test_secret_wrappers() {
        let sb = SecretBytes::new(vec![1, 2, 3]);
        assert_eq!(sb.as_slice(), &[1, 2, 3]);
        assert!(format!("{sb:?}").contains("REDACTED"));

        let ss = SecretString::new("secret".into());
        assert_eq!(ss.as_str(), "secret");
        assert!(format!("{ss:?}").contains("REDACTED"));
    }

    #[test]
    fn test_verify_error_branches() {
        let (_, public) = generate_keypair();
        let sig = hex::encode([0u8; 64]);

        // 1. Invalid public key hex
        assert!(verify_result("invalid", "j", "l", 0.0, 0, 0, &sig).is_err());

        // 2. Invalid public key length
        assert!(verify_result("001122", "j", "l", 0.0, 0, 0, &sig).is_err());

        // 3. Invalid signature hex
        assert!(verify_result(&public, "j", "l", 0.0, 0, 0, "invalid").is_err());

        // 4. Invalid signature length
        assert!(verify_result(&public, "j", "l", 0.0, 0, 0, "001122").is_err());
    }

    #[test]
    fn test_security_error_display() {
        assert!(format!("{}", SecurityError::Encoding("e".into())).contains("Encoding Error"));
        assert!(format!("{}", SecurityError::Key("k".into())).contains("Key Error"));
        assert!(format!("{}", SecurityError::Signature("s".into())).contains("Signature Error"));
    }
}
