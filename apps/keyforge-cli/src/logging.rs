// apps/keyforge-cli/src/logging.rs

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

use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub fn init_tracing() {
    // Set global propagator for context propagation
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,keyforge_cli=debug".into());

    let fmt_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stderr);

    if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() {
        if let Ok(exporter) = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()
        {
            let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
                .with_batch_exporter(exporter)
                .build();

            let tracer = provider.tracer("keyforge-cli");

            let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer)
                .with(telemetry_layer)
                .init();

            tracing::info!("🔭 Distributed Tracing Enabled (OTLP)");
        } else {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer)
                .init();
            tracing::warn!("⚠️ Failed to initialize OTLP exporter, falling back to local logging");
        }
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();

        tracing::info!("📝 Local Logging Enabled (Stderr)");
    }
}
