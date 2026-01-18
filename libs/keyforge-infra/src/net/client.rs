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
use keyforge_model::constants::{
    DEFAULT_CONNECT_TIMEOUT_SECS, DEFAULT_HIVE_URL, DEFAULT_REQUEST_TIMEOUT_SECS,
    DEFAULT_USER_AGENT,
};
use reqwest::{header, Client, RequestBuilder};
use std::time::Duration;

/// Configuration for the `HiveClient`.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// The base URL of the Hive API (Control Plane).
    pub api_url: String,
    /// The base URL of the Asset Server (Data Plane).
    pub asset_url: String,
    /// Optional secret key for authentication.
    pub secret: Option<String>,
    /// Request timeout (default: 30s).
    pub timeout: Duration,
    /// Connection timeout (default: 10s).
    pub connect_timeout: Duration,
    /// Custom User-Agent string.
    pub user_agent: String,
    /// Allow invalid/self-signed certificates (Dev Mode).
    pub accept_invalid_certs: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_url: DEFAULT_HIVE_URL.to_string(),
            asset_url: "http://localhost:3001".to_string(),
            secret: None,
            timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
            connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
            user_agent: DEFAULT_USER_AGENT.to_string(),
            accept_invalid_certs: false,
        }
    }
}

/// A specialized HTTP client for interacting with the `KeyForge` ecosystem.
#[derive(Clone, Debug)]
pub struct HiveClient {
    api_url: String,
    asset_url: String,
    inner: Client,
}

impl HiveClient {
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

        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(&config.user_agent)
                .map_err(|_| InfraError::Config("Invalid user agent characters".into()))?,
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .danger_accept_invalid_certs(config.accept_invalid_certs)
            .build()?;

        Ok(Self {
            api_url: normalize_url(&config.api_url),
            asset_url: normalize_url(&config.asset_url),
            inner: client,
        })
    }

    /// Expose inner client for low-level operations.
    #[must_use] 
    pub fn inner(&self) -> &Client {
        &self.inner
    }

    /// Construct a URL for the Control Plane (API).
    #[must_use] 
    pub fn url(&self, path: &str) -> String {
        format_url(&self.api_url, path)
    }

    /// Construct a URL for the Data Plane (Assets).
    #[must_use] 
    pub fn asset_url(&self, path: &str) -> String {
        format_url(&self.asset_url, path)
    }

    /// Starts a GET request to the API.
    pub fn get(&self, path: &str) -> RequestBuilder {
        self.inner.get(self.url(path))
    }

    /// Starts a POST request to the API.
    pub fn post(&self, path: &str) -> RequestBuilder {
        self.inner.post(self.url(path))
    }
}

fn normalize_url(url: &str) -> String {
    if url.ends_with('/') {
        url[..url.len() - 1].to_string()
    } else {
        url.to_string()
    }
}

fn format_url(base: &str, path: &str) -> String {
    if path.starts_with("http") {
        path.to_string()
    } else {
        format!("{}/{}", base, path.trim_start_matches('/'))
    }
}
