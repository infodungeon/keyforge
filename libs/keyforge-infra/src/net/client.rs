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
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:3000".to_string(),
            asset_url: "http://localhost:3001".to_string(),
            secret: None,
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            user_agent: "KeyForge-Client/0.9".to_string(),
        }
    }
}

/// A specialized HTTP client for interacting with the KeyForge ecosystem.
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
            .build()?;

        Ok(Self {
            api_url: normalize_url(&config.api_url),
            asset_url: normalize_url(&config.asset_url),
            inner: client,
        })
    }

    /// Expose inner client for low-level operations.
    pub fn inner(&self) -> &Client {
        &self.inner
    }

    /// Construct a URL for the Control Plane (API).
    pub fn url(&self, path: &str) -> String {
        format_url(&self.api_url, path)
    }

    /// Construct a URL for the Data Plane (Assets).
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
