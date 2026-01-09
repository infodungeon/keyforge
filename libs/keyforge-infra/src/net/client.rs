// libs/keyforge-infra/src/net/client.rs

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
use reqwest::{header, Client, RequestBuilder};
use std::time::Duration;

/// Configuration for the HiveClient.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// The base URL of the Hive server.
    pub base_url: String,
    /// Optional secret key for authentication.
    pub secret: Option<String>,
    /// Request timeout (default: 30s).
    pub timeout: Duration,
    /// Connection timeout (default: 10s).
    pub connect_timeout: Duration,
    /// Custom User-Agent string.
    pub user_agent: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8000".to_string(),
            secret: None,
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            user_agent: "KeyForge-Client/0.7".to_string(),
        }
    }
}

/// A specialized HTTP client for interacting with the KeyForge Hive API.
///
/// It handles base URL normalization, secret-based authentication, and
/// provides helpers for constructing requests to the Hive.
#[derive(Clone)]
pub struct HiveClient {
    base_url: String,
    inner: Client,
}

impl HiveClient {
    /// Creates a new `HiveClient` from the given configuration.
    ///
    /// If a secret is provided, it will be included in the `X-Keyforge-Secret` header
    /// for all requests.
    pub fn new(config: ClientConfig) -> InfraResult<Self> {
        let mut headers = header::HeaderMap::new();
        if let Some(s) = config.secret {
            if !s.is_empty() {
                let mut val = header::HeaderValue::from_str(&s)
                    .map_err(|_| InfraError::Config("Invalid secret key characters".into()))?;
                val.set_sensitive(true);
                headers.insert("X-Keyforge-Secret", val);
            }
        }

        // Standard User Agent
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(&config.user_agent)
                .map_err(|_| InfraError::Config("Invalid user agent characters".into()))?,
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .build()?;

        // Normalize URL (strip trailing slash)
        let base_url = config.base_url;
        let normalized_url = if base_url.ends_with('/') {
            base_url[..base_url.len() - 1].to_string()
        } else {
            base_url
        };

        Ok(Self {
            base_url: normalized_url,
            inner: client,
        })
    }

    /// Helper to construct a full URL
    pub fn url(&self, path: &str) -> String {
        if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}/{}", self.base_url, path.trim_start_matches('/'))
        }
    }

    /// Expose inner client for low-level operations (like ensure_file)
    pub fn inner(&self) -> &Client {
        &self.inner
    }

    /// Starts a GET request to the specified path relative to the base URL.
    pub fn get(&self, path: &str) -> RequestBuilder {
        self.inner.get(self.url(path))
    }

    /// Starts a POST request to the specified path relative to the base URL.
    pub fn post(&self, path: &str) -> RequestBuilder {
        self.inner.post(self.url(path))
    }
}
