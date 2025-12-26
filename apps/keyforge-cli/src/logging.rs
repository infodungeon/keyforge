use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing() {
    // Set global propagator for context propagation
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,keyforge_cli=debug".into());

    let fmt_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stderr);

    if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()
            .expect("Failed to create OTLP exporter");

        let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .build();

        use opentelemetry::trace::TracerProvider;
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

        tracing::info!("📝 Local Logging Enabled (Stderr)");
    }
}
