// libs/keyforge-infra/src/fs/lock.rs

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

use crate::error::{InfraError, InfraResult};
use fs2::FileExt;
use keyforge_model::constants::{LOCK_INITIAL_DELAY_MS, LOCK_MAX_ATTEMPTS};
use std::fs::File;
use std::path::Path;
use std::time::Duration;

/// A process-level lock that ensures only one instance of `KeyForge` is accessing the workspace.
///
/// This uses mandatory file locking (via `fs2`) on a lockfile within the workspace root.
#[derive(Debug)]
pub struct WorkspaceLock {
    file: File,
}

impl WorkspaceLock {
    /// Attempts to acquire an exclusive lock on the specified path.
    ///
    /// # Errors
    /// Returns `InfraError::LockError` if the lock is already held by another process.
    pub fn acquire(path: &Path) -> InfraResult<Self> {
        let file = File::open(path).map_err(InfraError::Io)?;

        // Retry loop with exponential backoff
        let mut attempts = 0;
        let mut delay = Duration::from_millis(LOCK_INITIAL_DELAY_MS);

        loop {
            match file.try_lock_exclusive() {
                Ok(()) => return Ok(Self { file }),
                Err(e) => {
                    attempts += 1;
                    if attempts >= LOCK_MAX_ATTEMPTS {
                        return Err(InfraError::LockError(format!(
                            "Failed to acquire lock on {} after {attempts} attempts: {e}",
                            path.display()
                        )));
                    }
                    std::thread::sleep(delay);
                    delay = (delay * 2).min(Duration::from_secs(1));
                }
            }
        }
    }

    /// Explicitly releases the lock.
    ///
    /// The lock is also automatically released when the `WorkspaceLock` instance is dropped.
    ///
    /// # Errors
    ///
    /// Returns `InfraError::Io` if the file cannot be unlocked.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_workspace_lock_exclusivity() {
        let temp = tempfile::tempdir().unwrap();
        let lock_path = temp.path().join("workspace.lock");
        fs::File::create(&lock_path).unwrap();

        let lock_a = WorkspaceLock::acquire(&lock_path);
        assert!(lock_a.is_ok());

        let lock_b = WorkspaceLock::acquire(&lock_path);
        assert!(lock_b.is_err());

        drop(lock_a);

        let lock_c = WorkspaceLock::acquire(&lock_path);
        assert!(lock_c.is_ok());
    }

    #[test]
    fn test_workspace_lock_release() {
        let temp = tempfile::tempdir().unwrap();
        let lock_path = temp.path().join("workspace.lock");
        fs::File::create(&lock_path).unwrap();

        let lock = WorkspaceLock::acquire(&lock_path).unwrap();
        lock.release().unwrap();
        
        let lock2 = WorkspaceLock::acquire(&lock_path);
        assert!(lock2.is_ok());
    }
}
