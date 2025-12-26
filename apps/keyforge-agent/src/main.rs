use clap::Parser;
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use std::path::PathBuf;
use tokio::sync::broadcast;
use tracing::{error, info};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "http://localhost:3000")]
    hive: String,

    #[arg(long, default_value_t = 4)]
    cores: usize,

    #[arg(long, default_value = "data")]
    data_dir: PathBuf,
}

#[tokio::main]
async fn main() {
    keyforge_agent::logging::init_tracing();
    let args = Args::parse();

    info!("agent starting");
    info!(hive_url = %args.hive, "connecting to hive");
    info!(data_dir = ?args.data_dir, "data directory configured");

    // 1. Identity Management
    let signing_key = match load_or_create_identity() {
        Ok(k) => k,
        Err(e) => {
            error!(error = %e, "fatal: identity error");
            std::process::exit(1);
        }
    };

    let public_key = VerifyingKey::from(&signing_key);
    // Task 5: Use first 8 chars of PK hash instead of raw hex-encoded PK
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(public_key.to_bytes());
    let pk_hash = hex::encode(hasher.finalize());
    let node_id = format!("agent-{}", &pk_hash[0..8]);

    // 2. Signal Handling
    // Task 48: Increase broadcast channel capacity above 1
    let (tx, rx) = broadcast::channel(16);
    #[cfg(unix)]
    let mut sig_usr1 =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::user_defined1())
            .expect("failed to register SIGUSR1");
    #[cfg(not(unix))]
    let mut sig_usr1 = std::future::pending::<()>();

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("received ctrl-c, initiating shutdown");
                let _ = tx_clone.send(());
            }
            _ = sig_usr1.recv() => {
                info!("received SIGUSR1, initiating graceful drain");
                let _ = tx_clone.send(());
            }
        }
    });

    // 3. Run Worker
    keyforge_agent::run_worker(args.hive, node_id, None, signing_key, args.data_dir, rx).await;

    info!("agent exited cleanly");
}

/// Loads the agent's identity key, decrypting it if it exists, or generating a new one.
///
/// # Security
/// - The identity is encrypted at rest using the `age` crate.
/// - The passphrase is derived from a machine-specific ID.
/// - File permissions are hardened to 0600 on Unix systems.
fn load_or_create_identity() -> Result<SigningKey, String> {
    // P2 FIX: Use XDG config directory instead of home root
    let mut path = dirs::config_dir().ok_or("could not find config directory")?;
    path.push("keyforge");

    if !path.exists() {
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("failed to create config dir: {}", e))?;
    }

    path.push("agent.key.age");

    // Task 9: Deriving machine ID
    let passphrase = machine_id_timeout_safe()
        .expect("Fatal: Could not derive machine ID. Secure fallback unavailable.");

    if path.exists() {
        let file =
            std::fs::File::open(&path).map_err(|e| format!("failed to open key file: {}", e))?;
        let decryptor =
            age::Decryptor::new(file).map_err(|e| format!("age decryptor error: {}", e))?;

        // age 0.11 expects a SecretString which might be a specific secrecy type
        let identity = age::scrypt::Identity::new(passphrase.clone().into());
        let mut reader = decryptor
            .decrypt(std::iter::once(&identity as &dyn age::Identity))
            .map_err(|e| format!("decryption failed: {}", e))?;

        let mut decrypted = Vec::new();
        use std::io::Read;
        reader
            .read_to_end(&mut decrypted)
            .map_err(|e| format!("failed to read decrypted key: {}", e))?;

        let array: [u8; 32] = decrypted
            .try_into()
            .map_err(|_| "invalid key file length (expected 32 bytes)")?;

        info!(path = ?path, "loaded encrypted identity");
        Ok(SigningKey::from_bytes(&array))
    } else {
        let mut csprng = OsRng;
        let key = SigningKey::generate(&mut csprng);

        // Task 9 & 10: Encrypt and harden permissions
        let encryptor = age::Encryptor::with_user_passphrase(passphrase.into());

        let mut output = Vec::new();
        let mut writer = encryptor
            .wrap_output(&mut output)
            .map_err(|e| format!("failed to initialize age writer: {}", e))?;
        use std::io::Write;
        writer
            .write_all(&key.to_bytes())
            .map_err(|e| format!("failed to write to age writer: {}", e))?;
        writer
            .finish()
            .map_err(|e| format!("failed to finish age encryption: {}", e))?;

        // Create file with hardened permissions (0600 on Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&path)
                .map_err(|e| format!("failed to create hardened key file: {}", e))?;
            file.write_all(&output)
                .map_err(|e| format!("failed to save encrypted key: {}", e))?;
        }
        #[cfg(not(unix))]
        {
            std::fs::write(&path, &output)
                .map_err(|e| format!("failed to save encrypted key: {}", e))?;
        }

        info!(path = ?path, "generated new encrypted identity");
        Ok(key)
    }
}

/// Retrieves a stable machine-specific ID.
fn machine_id_timeout_safe() -> Result<String, String> {
    machine_uid::get().map_err(|e| {
        format!(
            "Security Requirement: Could not derive unique machine ID: {}",
            e
        )
    })
}
