// libs/keyforge-infra/src/fs/io.rs

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
use std::io::{Read, Write};
use std::path::Path;
use tempfile::NamedTempFile;

/// Performs an atomic write by writing to a temporary file and then moving it to the target path.
///
/// This ensures that the target file is never in a partially written state, even if the
/// process crashes or power is lost during the write.
///
/// # Errors
///
/// Returns `InfraError` if directory creation, writing, or file persistence fails.
pub fn atomic_write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, content: C) -> InfraResult<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(InfraError::Io)?;
    }
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temp_file = NamedTempFile::new_in(dir).map_err(InfraError::Io)?;
    temp_file
        .write_all(content.as_ref())
        .map_err(InfraError::Io)?;
    temp_file.flush().map_err(InfraError::Io)?;
    temp_file
        .persist(path)
        .map_err(|e| InfraError::Io(e.error))?;

    // --- DURABILITY: Task-sec-015 ---
    // Perform fsync on the parent directory to ensure the directory entry is durable.
    // This protects against 0-byte files or missing entries after a power failure.
    if let Some(parent) = path.parent() {
        if let Ok(dir) = std::fs::File::open(parent) {
            let _ = dir.sync_all();
        }
    }

    Ok(())
}

/// Reads a file's content into a string, but only if its size is below the specified limit.
///
/// This is a security measure to prevent memory exhaustion when reading untrusted input files.
///
/// # Errors
///
/// Returns `InfraError` if the file cannot be opened, read, or if it exceeds the size limit.
pub fn read_to_string_limited<P: AsRef<Path>>(path: P, limit_bytes: u64) -> InfraResult<String> {
    let path = path.as_ref();
    let file = std::fs::File::open(path).map_err(InfraError::Io)?;
    let mut reader = file.take(limit_bytes + 1);
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).map_err(InfraError::Io)?;

    if buffer.len() as u64 > limit_bytes {
        return Err(InfraError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("File exceeds size limit of {limit_bytes} bytes"),
        )));
    }
    Ok(buffer)
}
