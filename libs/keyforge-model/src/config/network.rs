// libs/keyforge-model/src/config/network.rs

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

use crate::validator::Validator;
use zeroize::Zeroize;

/// Configuration for Cross-Origin Resource Sharing (CORS).
#[derive(Debug, Clone, Default, Zeroize)]
pub struct CorsConfig {
    /// Allowed origins. Can be a comma-separated list of URLs or "*" for permissive access.
    /// If empty, defaults to restricted mode (e.g. localhost only).
    pub allowed_origins: String,
}

impl Validator for CorsConfig {
    fn validate(&self) -> Result<(), String> {
        // Validation could check if URLs are well-formed, but "*" and empty are valid.
        Ok(())
    }
}
