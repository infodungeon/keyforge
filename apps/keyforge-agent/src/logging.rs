// apps/keyforge-agent/src/logging.rs

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


#![deny(clippy::expect_used)]

use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initializes the tracing system for the agent.
///
/// It supports both local stdout logging and distributed tracing via OTLP if
/// the `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable is set.
pub fn init_tracing() {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,keyforge_agent=debug".into());

    let fmt_layer = tracing_subscriber::fmt::layer();

    if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() {
        match opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()
        {
            Ok(exporter) => {
                let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
                    .with_batch_exporter(exporter)
                    .build();

                use opentelemetry::trace::TracerProvider;
                let tracer = provider.tracer("keyforge-agent");

                let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

                tracing_subscriber::registry()
                    .with(filter)
                    .with(fmt_layer)
                    .with(telemetry_layer)
                    .init();

                // Task 27: Structured logging
                tracing::info!(mode = "otlp", "distributed tracing enabled");
            }
            Err(e) => {
                tracing_subscriber::registry()
                    .with(filter)
                    .with(fmt_layer)
                    .init();
                // Task 27: Structured logging
                tracing::warn!(error = %e, "failed to create OTLP exporter, falling back to local");
            }
        }
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();

        // Task 27: Structured logging
        tracing::info!(mode = "stdout", "local logging enabled");
    }
}
