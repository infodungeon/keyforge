use crate::error::{InfraError, InfraResult};
use fs2::FileExt;
use std::fs::File;
use std::path::Path;

pub struct WorkspaceLock {
    file: File,
}

impl WorkspaceLock {
    pub fn acquire(path: &Path) -> InfraResult<Self> {
        let file = File::open(path).map_err(InfraError::Io)?;

        // Try to acquire an exclusive lock
        file.try_lock_exclusive().map_err(|e| {
            InfraError::LockError(format!("Failed to acquire lock on {:?}: {}", path, e))
        })?;

        Ok(Self { file })
    }

    pub fn release(&self) -> InfraResult<()> {
        self.file.unlock().map_err(InfraError::Io)?;
        Ok(())
    }
}

impl Drop for WorkspaceLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}
