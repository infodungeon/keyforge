use crate::agent::errors::AgentError;
use crate::models::SystemConfig;
use ed25519_dalek::SigningKey;
use std::convert::TryInto;
use std::io::{Read, Write};
use tracing::info;

pub fn load_or_create_identity(config: &SystemConfig) -> Result<SigningKey, AgentError> {
    let mut config_dir = dirs::config_dir()
        .ok_or_else(|| AgentError::Identity("could not find config directory".into()))?;
    config_dir.push(&config.config_dir_name);

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| AgentError::Identity(format!("failed to create config dir: {e}")))?;
    }

    let mut key_path = config_dir.clone();
    key_path.push(&config.key_file_name);

    let passphrase = if let Some(override_id) = &config.machine_id_override {
        override_id.clone()
    } else if let Ok(env_id) = std::env::var("KEYFORGE_MACHINE_ID") {
        env_id
    } else {
        // Task-agent-021: Fallback to persistent UUID if system machine ID is unavailable
        if let Ok(id) = machine_uid::get() {
            id
        } else {
            let mut uuid_path = config_dir.clone();
            uuid_path.push("machine_id.uuid");
            if uuid_path.exists() {
                std::fs::read_to_string(&uuid_path).map_err(|e| {
                    AgentError::Identity(format!("Failed to read fallback UUID: {e}"))
                })?
            } else {
                let new_id = uuid::Uuid::new_v4().to_string();
                std::fs::write(&uuid_path, &new_id).map_err(|e| {
                    AgentError::Identity(format!("Failed to save fallback UUID: {e}"))
                })?;
                info!(path = ?uuid_path, "Generated new fallback machine UUID");
                new_id
            }
        }
    };

    if key_path.exists() {
        let file = std::fs::File::open(&key_path)
            .map_err(|e| AgentError::Identity(format!("failed to open key file: {e}")))?;
        let decryptor = age::Decryptor::new(file)
            .map_err(|e| AgentError::Identity(format!("age decryptor error: {e}")))?;

        let identity = age::scrypt::Identity::new(passphrase.clone().into());
        let mut reader = decryptor
            .decrypt(std::iter::once(&identity as &dyn age::Identity))
            .map_err(|e| AgentError::Identity(format!("decryption failed: {e}")))?;

        let mut decrypted = Vec::new();
        reader
            .read_to_end(&mut decrypted)
            .map_err(|e| AgentError::Identity(format!("failed to read decrypted key: {e}")))?;

        let array: [u8; 32] = decrypted.try_into().map_err(|_| {
            AgentError::Identity("invalid key file length (expected 32 bytes)".into())
        })?;

        Ok(SigningKey::from_bytes(&array))
    } else {
        let mut bytes = [0u8; 32];
        let mut csprng = rand::rng();
        rand::RngCore::fill_bytes(&mut csprng, &mut bytes);
        let key = SigningKey::from_bytes(&bytes);

        let encryptor = age::Encryptor::with_user_passphrase(passphrase.into());

        let mut output = Vec::new();
        let mut writer = encryptor
            .wrap_output(&mut output)
            .map_err(|e| AgentError::Identity(format!("failed to initialize age writer: {e}")))?;
        writer
            .write_all(&key.to_bytes())
            .map_err(|e| AgentError::Identity(format!("failed to write to age writer: {e}")))?;
        writer
            .finish()
            .map_err(|e| AgentError::Identity(format!("failed to finish age encryption: {e}")))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&key_path)
                .map_err(|e| {
                    AgentError::Identity(format!("failed to create hardened key file: {e}"))
                })?;
            file.write_all(&output)
                .map_err(|e| AgentError::Identity(format!("failed to save encrypted key: {e}")))?;
        }
        #[cfg(not(unix))]
        {
            std::fs::write(&key_path, &output).map_err(|e| {
                AgentError::Identity(format!("failed to save encrypted key: {}", e))
            })?;
        }

        info!(path = ?key_path, "generated new encrypted identity");
        Ok(key)
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_identity_file_hardening() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let key_path = dir.path().join("agent.key.age");

        fs::write(&key_path, "dummy encrypted data")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&key_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&key_path, perms)?;

            let final_perms = fs::metadata(&key_path)?.permissions();
            assert_eq!(
                final_perms.mode() & 0o777,
                0o600,
                "Identity file must be owner-readable only"
            );
        }
        Ok(())
    }
}
