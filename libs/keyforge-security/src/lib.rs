// libs/keyforge-security/src/lib.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
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
use pasetors::claims::{Claims, ClaimsValidationRules};
use pasetors::keys::SymmetricKey;
use pasetors::token::UntrustedToken;
use pasetors::version4::V4;
use pasetors::Local;
use sha2::{Digest, Sha256};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Errors that can occur during security operations.
#[derive(Error, Debug)]
pub enum SecurityError {
    /// Failed to encode or decode a cryptographic primitive (e.g., hex string parsing).
    #[error("Encoding Error: {0}")]
    Encoding(#[from] hex::FromHexError),
    /// An issue with a dalek secret or public key.
    #[error("Dalek Key Error: {0}")]
    Dalek(#[from] ed25519_dalek::SignatureError),
    /// A general key-related error.
    #[error("Key Error: {0}")]
    Key(String),
    /// Failed to create or verify a digital signature.
    #[error("Signature Error: {0}")]
    Signature(String),
    /// Error during token generation or verification.
    #[error("Token Error: {0}")]
    Token(String),
}

/// A specialized Result type for security operations.
pub type SecurityResult<T> = Result<T, SecurityError>;

/// A wrapper for sensitive byte arrays that ensures data is zeroed out on drop.
///
/// Use this for storing raw keys or other sensitive binary data in memory to
/// mitigate the risk of data leakage after the value is no longer needed.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretBytes {
    data: Vec<u8>,
}

impl std::fmt::Debug for SecretBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretBytes")
            .field("data", &"***REDACTED***")
            .finish()
    }
}

impl SecretBytes {
    /// Wraps a vector of bytes in a `SecretBytes` container.
    #[must_use]
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
    /// Returns a reference to the protected byte slice.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

/// A wrapper for sensitive strings that ensures data is zeroed out on drop.
///
/// Use this for storing passwords, API keys, or other sensitive text in memory.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretString {
    inner: String,
}

impl std::fmt::Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretString")
            .field("inner", &"***REDACTED***")
            .finish()
    }
}

impl SecretString {
    /// Wraps a string in a `SecretString` container.
    #[must_use]
    pub fn new(s: String) -> Self {
        Self { inner: s }
    }
    /// Returns a reference to the protected string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.inner
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

fn build_payload(
    job_id: &str,
    layout: &str,
    score_fixed: i64,
    timestamp: u64,
    nonce: u64,
) -> Vec<u8> {
    let mut hasher = Sha256::new();

    // Domain Separator to prevent cross-protocol attacks
    hasher.update(b"KeyForge-Result-v1");

    hasher.update((job_id.len() as u64).to_le_bytes());
    hasher.update(job_id.as_bytes());

    hasher.update((layout.len() as u64).to_le_bytes());
    hasher.update(layout.as_bytes());

    let mut payload = Vec::with_capacity(128);
    payload.extend_from_slice(&hasher.finalize());
    payload.extend_from_slice(&score_fixed.to_le_bytes());
    payload.extend_from_slice(&timestamp.to_le_bytes());
    payload.extend_from_slice(&nonce.to_le_bytes());

    payload
}

/// Signs an optimization result using a hex-encoded Ed25519 secret key and a fixed-point score.
///
/// # Errors
/// Returns `SecurityError::Encoding` if the secret key is not a valid 64-character hex string.
pub fn sign_result_fixed(
    secret_hex: &str,
    job_id: &str,
    layout: &str,
    score_fixed: i64,
    timestamp: u64,
    nonce: u64,
) -> SecurityResult<String> {
    let secret_hex = secret_hex.trim();
    let mut key_buf = [0u8; 32];
    hex::decode_to_slice(secret_hex, &mut key_buf)?;

    let signing_key = SigningKey::from_bytes(&key_buf);
    key_buf.zeroize();

    sign_result_direct(&signing_key, job_id, layout, score_fixed, timestamp, nonce)
}

/// Signs an optimization result using a hex-encoded Ed25519 secret key.
///
/// The signature covers the `job_id`, `layout`, `score`, `timestamp`, and `nonce`.
/// Returns the hex-encoded signature string.
///
/// # Errors
/// Returns `SecurityError` if the secret key is not a valid 64-character hex string.
pub fn sign_result(
    secret_hex: &str,
    job_id: &str,
    layout: &str,
    score: f32,
    timestamp: u64,
    nonce: u64,
) -> SecurityResult<String> {
    // SAFETY: TYPE-001 Exception: Physics-aware conversion to scaled fixed-point.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    let score_fixed = (f64::from(score) * 1_000_000.0).round() as i64; // SAFETY: TYPE-001
    sign_result_fixed(secret_hex, job_id, layout, score_fixed, timestamp, nonce)
}

/// Signs an optimization result using a pre-loaded `SigningKey`.
///
/// This is the "direct" version of `sign_result` that avoids re-parsing the key.
/// The signature covers the `job_id`, `layout`, `score_fixed`, `timestamp`, and `nonce`.
/// Returns the hex-encoded signature string.
///
/// # Errors
/// This function is currently infallible but returns a `Result` for consistency with the
/// `sign_result` API and to support future cryptographic backends that may fail.
pub fn sign_result_direct(
    signing_key: &SigningKey,
    job_id: &str,
    layout: &str,
    score_fixed: i64,
    timestamp: u64,
    nonce: u64,
) -> SecurityResult<String> {
    let payload = build_payload(job_id, layout, score_fixed, timestamp, nonce);
    let signature = signing_key.sign(&payload);
    Ok(hex::encode(signature.to_bytes()))
}

/// Verifies a signed optimization result against a hex-encoded Ed25519 public key and a fixed-point score.
///
/// # Errors
/// Returns `SecurityError` if any of the hex inputs are malformed or keys are invalid lengths.
pub fn verify_result_fixed(
    public_hex: &str,
    job_id: &str,
    layout: &str,
    score_fixed: i64,
    timestamp: u64,
    nonce: u64,
    signature_hex: &str,
) -> SecurityResult<bool> {
    let public_hex = public_hex.trim();
    let signature_hex = signature_hex.trim();

    let public_bytes = hex::decode(public_hex)?;

    let verifying_key = VerifyingKey::from_bytes(
        public_bytes
            .as_slice()
            .try_into()
            .map_err(|_| SecurityError::Key("Invalid public key length".into()))?,
    )?;

    let signature_bytes = hex::decode(signature_hex)?;

    let signature = Signature::from_bytes(
        signature_bytes
            .as_slice()
            .try_into()
            .map_err(|_| SecurityError::Key("Invalid signature length".into()))?,
    );

    let payload = build_payload(job_id, layout, score_fixed, timestamp, nonce);

    match verifying_key.verify(&payload, &signature) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
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
    // SAFETY: TYPE-001 Exception: Physics-aware conversion to scaled fixed-point.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    let score_fixed = (f64::from(score) * 1_000_000.0).round() as i64; // SAFETY: TYPE-001
    verify_result_fixed(
        public_hex,
        job_id,
        layout,
        score_fixed,
        timestamp,
        nonce,
        signature_hex,
    )
}

/// Creates a new PASETO V4.Local token for the given subject using a 32-byte secret key.
///
/// # Errors
/// Returns `SecurityError::Token` if token generation fails.
pub fn create_paseto_token(secret: &[u8], subject: &str, ttl_secs: u64) -> SecurityResult<String> {
    let sk = SymmetricKey::<V4>::from(secret)
        .map_err(|e| SecurityError::Key(format!("Invalid symmetric key: {e}")))?;

    let mut claims =
        Claims::new().map_err(|e| SecurityError::Token(format!("Claims error: {e}")))?;
    claims
        .subject(subject)
        .map_err(|e| SecurityError::Token(format!("Claims subject error: {e}")))?;

    let ttl_i64 =
        i64::try_from(ttl_secs).map_err(|_| SecurityError::Token("TTL exceeds max i64".into()))?;
    let expiration = chrono::Utc::now() + chrono::Duration::seconds(ttl_i64);
    claims
        .expiration(&expiration.to_rfc3339())
        .map_err(|e| SecurityError::Token(format!("Claims expiration error: {e}")))?;

    pasetors::local::encrypt(&sk, &claims, None, None)
        .map_err(|e| SecurityError::Token(format!("Encryption error: {e}")))
}

/// Verifies a PASETO V4.Local token and returns the subject (`node_id`).
///
/// # Errors
/// Returns `SecurityError::Token` if the token is invalid or expired.
pub fn verify_paseto_token(secret: &[u8], token: &str) -> SecurityResult<String> {
    let sk = SymmetricKey::<V4>::from(secret)
        .map_err(|e| SecurityError::Key(format!("Invalid symmetric key: {e}")))?;

    let validation_rules = ClaimsValidationRules::new();
    let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)
        .map_err(|e| SecurityError::Token(format!("Token format error: {e}")))?;

    let trusted_token =
        pasetors::local::decrypt(&sk, &untrusted_token, &validation_rules, None, None)
            .map_err(|e| SecurityError::Token(format!("Decryption error: {e}")))?;

    trusted_token
        .payload_claims()
        .ok_or_else(|| SecurityError::Token("Missing claims in token".into()))?
        .get_claim("sub")
        .ok_or_else(|| SecurityError::Token("Missing subject in token".into()))?
        .as_str()
        .ok_or_else(|| SecurityError::Token("Subject is not a string".into()))
        .map(std::string::ToString::to_string)
}

/// Generates a cryptographically secure random 64-bit nonce.
///
/// Used to prevent replay attacks in signed messages.
#[must_use]
pub fn generate_nonce() -> u64 {
    use rand::Rng;
    rand::rng().random()
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify_happy_path() -> anyhow::Result<()> {
        let (secret, public) = generate_keypair();
        let job_id = "job-123";
        let layout = "qwerty...";
        let score = 98.6;
        let timestamp = 1_234_567_890;
        let nonce = generate_nonce();

        let sig = sign_result(&secret, job_id, layout, score, timestamp, nonce)?;
        let valid = verify_result(&public, job_id, layout, score, timestamp, nonce, &sig)?;

        assert!(valid, "Signature should verify correctly");
        Ok(())
    }

    #[test]
    fn test_sign_with_whitespace() -> anyhow::Result<()> {
        let (secret, public) = generate_keypair();
        // Add whitespace to secret
        let spaced_secret = format!("  {secret}  ");

        let sig = sign_result(&spaced_secret, "job", "layout", 1.0, 0, 0)?;
        let valid = verify_result(&public, "job", "layout", 1.0, 0, 0, &sig)?;
        assert!(valid, "Detailed whitespace should be trimmed");
        Ok(())
    }

    #[test]
    fn test_verify_with_whitespace() -> anyhow::Result<()> {
        let (secret, public) = generate_keypair();
        let sig = sign_result(&secret, "job", "layout", 1.0, 0, 0)?;

        let spaced_public = format!("\n{public}\t");
        let spaced_sig = format!(" {sig} ");

        let valid = verify_result(&spaced_public, "job", "layout", 1.0, 0, 0, &spaced_sig)?;
        assert!(
            valid,
            "Public key and signature whitespace should be trimmed"
        );
        Ok(())
    }

    #[test]
    fn test_invalid_hex() -> anyhow::Result<()> {
        let res = sign_result("not-hex-at-all", "job", "layout", 1.0, 0, 0);
        assert!(matches!(res, Err(SecurityError::Encoding(_))));
        Ok(())
    }

    #[test]
    fn test_verify_tampered_payload() -> anyhow::Result<()> {
        let (secret, public) = generate_keypair();
        let sig = sign_result(&secret, "job", "layout", 1.0, 0, 0)?;

        // Check with different score
        let valid = verify_result(&public, "job", "layout", 99.0, 0, 0, &sig)?;
        assert!(!valid, "Tampered payload should verify as false");
        Ok(())
    }

    #[test]
    fn test_secret_wrappers() -> anyhow::Result<()> {
        let sb = SecretBytes::new(vec![1, 2, 3]);
        assert_eq!(sb.as_slice(), &[1, 2, 3]);
        assert!(format!("{sb:?}").contains("REDACTED"));

        let ss = SecretString::new("secret".into());
        assert_eq!(ss.as_str(), "secret");
        assert!(format!("{ss:?}").contains("REDACTED"));
        Ok(())
    }

    #[test]
    fn test_verify_error_branches() -> anyhow::Result<()> {
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
        Ok(())
    }

    #[test]
    fn test_security_error_display() -> anyhow::Result<()> {
        let hex_err = hex::FromHexError::InvalidHexCharacter { c: 'g', index: 0 };
        assert!(format!("{}", SecurityError::Encoding(hex_err)).contains("Encoding Error"));
        assert!(format!("{}", SecurityError::Key("k".into())).contains("Key Error"));
        assert!(format!("{}", SecurityError::Signature("s".into())).contains("Signature Error"));
        Ok(())
    }
}
