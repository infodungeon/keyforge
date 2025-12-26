// ===== keyforge/crates/keyforge-workspace/src/client.rs =====
use reqwest::{header, Client, RequestBuilder};
use std::time::Duration;

#[derive(Clone)]
pub struct HiveClient {
    base_url: String,
    inner: Client,
}

impl HiveClient {
    pub fn new(base_url: String, secret: Option<String>) -> Result<Self, String> {
        let mut headers = header::HeaderMap::new();
        if let Some(s) = secret {
            if !s.is_empty() {
                let mut val = header::HeaderValue::from_str(&s).map_err(|e| e.to_string())?;
                val.set_sensitive(true);
                headers.insert("X-Keyforge-Secret", val);
            }
        }

        // Standard User Agent
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("KeyForge-Client/0.7"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        // Normalize URL (strip trailing slash)
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

    pub fn get(&self, path: &str) -> RequestBuilder {
        self.inner.get(self.url(path))
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        self.inner.post(self.url(path))
    }
}
