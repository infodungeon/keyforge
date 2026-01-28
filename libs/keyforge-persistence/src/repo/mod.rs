// libs/keyforge-persistence/src/repo/mod.rs

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

/// Implementation of the user-specific data repository.
pub mod user_repo;

/// Repository for community layout submissions and shared assets.
pub mod community_repo;

/// Repository for biometric typing profiles and timing data.
pub mod biometric_repo;

/// Repository for research metrics and token tracking.
pub mod research_repo;

/// Repository for managing optimization and analysis sessions.
pub mod session_repo;
