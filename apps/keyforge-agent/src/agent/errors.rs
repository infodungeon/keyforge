// apps/keyforge-agent/src/agent/errors.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Identity Error: {0}")]
    Identity(String),

    #[error("Hardware Detection Error: {0}")]
    Hardware(String),

    #[error("Calibration Error: {0}")]
    Calibration(String),

    #[error("Network Error: {0}")]
    Network(String),

    #[error("Internal Error: {0}")]
    Internal(String),

    #[error("Resource Error: {0}")]
    Resource(String),
}

pub type AgentResult<T> = Result<T, AgentError>;
