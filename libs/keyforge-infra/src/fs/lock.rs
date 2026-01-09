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
use std::fs::File;
use std::path::Path;

/// A process-level lock that ensures only one instance of KeyForge is accessing the workspace.
///
/// This uses mandatory file locking (via `fs2`) on a lockfile within the workspace root.
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

        // Retry loop to handle race conditions (e.g. restart)
        let mut attempts = 0;
        loop {
            match file.try_lock_exclusive() {
                Ok(_) => return Ok(Self { file }),
                Err(e) => {
                    attempts += 1;
                    if attempts >= 5 {
                        return Err(InfraError::LockError(format!(
                            "Failed to acquire lock on {:?} after 5 attempts: {}",
                            path, e
                        )));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }

    /// Explicitly releases the lock.
    ///
    /// The lock is also automatically released when the `WorkspaceLock` instance is dropped.
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
