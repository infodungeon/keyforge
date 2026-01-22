// Copyright (c) 2025 KeyForge Contributors
//
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

use thiserror::Error;

/// Errors that can occur during the export process.
#[derive(Error, Debug)]
pub enum ExportError {
    /// The number of keys exceeds the firmware limit.
    #[error("Too many keys for export (Limit: {0})")]
    TooManyKeys(usize),

    /// The generated output exceeds the maximum size limit.
    #[error("Output size limit exceeded")]
    OutputSizeLimitExceeded,

    /// The layout data is empty or invalid.
    #[error("Invalid layout: {0}")]
    InvalidLayout(String),

    /// Serialization error.
    #[error("Serialization failed: {0}")]
    SerializationFailed(#[from] serde_json::Error),

    /// Internal error with a custom message.
    #[error("Internal export error: {0}")]
    Internal(String),
}

/// A specialized Result type for export operations.
pub type ExportResult<T> = std::result::Result<T, ExportError>;
