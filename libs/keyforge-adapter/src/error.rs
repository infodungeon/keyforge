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

#[derive(Debug, Error, PartialEq)]
pub enum AdapterError {
    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Unknown key token: {0}")]
    UnknownToken(String),

    #[error("Layout string exceeds maximum length of {0}")]
    LayoutTooLong(usize),
}

pub type AdapterResult<T> = Result<T, AdapterError>;
