// apps/keyforge-agent/src/logging.rs

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


#![deny(clippy::expect_used)]

use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Logging output modes.
#[derive(Debug)]
pub enum LogMode {
    /// Standard human-readable logs to stdout (for Worker/Daemon).
    Standard,
    /// Structured JSON logs to stderr (for Sidecar/Run mode).
    JsonStderr,
}

/// Initializes the tracing system for the agent.
pub fn init_tracing(default_filter: &str, mode: &LogMode) {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_filter));

    match mode {
        LogMode::Standard => {
            let fmt_layer = tracing_subscriber::fmt::layer();
            // Check for OTLP only in Standard mode
            if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() {
                if let Ok(exporter) = opentelemetry_otlp::SpanExporter::builder().with_tonic().build() {
                    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
                        .with_batch_exporter(exporter)
                        .build();
                    let tracer = opentelemetry::trace::TracerProvider::tracer(&provider, "keyforge-agent");
                    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
                    
                    tracing_subscriber::registry()
                        .with(filter)
                        .with(fmt_layer)
                        .with(telemetry_layer)
                        .init();
                    return;
                }
            }
            // Fallback
            tracing_subscriber::registry().with(filter).with(fmt_layer).init();
        },
        LogMode::JsonStderr => {
            // Write JSON to stderr to keep stdout clean for results
            let fmt_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_writer(std::io::stderr);
            
            tracing_subscriber::registry().with(filter).with(fmt_layer).init();
        }
    }
}
