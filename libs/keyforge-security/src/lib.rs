use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Encoding Error: {0}")]
    Encoding(String),
    #[error("Key Error: {0}")]
    Key(String),
    #[error("Signature Error: {0}")]
    Signature(String),
}

pub type SecurityResult<T> = Result<T, SecurityError>;

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(s: String) -> Self {
        Self(s)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

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

pub fn generate_nonce() -> u64 {
    use rand::RngCore;
    OsRng.next_u64()
}
