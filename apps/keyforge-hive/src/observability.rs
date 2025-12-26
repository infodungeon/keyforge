use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use once_cell::sync::Lazy;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use serde::Serialize;
use std::collections::VecDeque;
use std::env;
use std::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

// --- Log Capture ---

#[derive(Clone, Serialize, Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

pub static LOG_BUFFER: Lazy<Mutex<VecDeque<LogEntry>>> =
    Lazy::new(|| Mutex::new(VecDeque::with_capacity(50)));

pub fn get_recent_logs() -> Vec<LogEntry> {
    if let Ok(buffer) = LOG_BUFFER.lock() {
        buffer.iter().cloned().collect()
    } else {
        vec![]
    }
}

struct LogCaptureLayer;

impl<S> Layer<S> for LogCaptureLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        // Capture WARN and ERROR
        if *metadata.level() <= tracing::Level::WARN {
            let mut visitor = MessageVisitor::new();
            event.record(&mut visitor);

            let entry = LogEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                level: metadata.level().to_string(),
                message: visitor.message,
            };

            if let Ok(mut buffer) = LOG_BUFFER.lock() {
                if buffer.len() >= 50 {
                    buffer.pop_front();
                }
                buffer.push_back(entry);
            }
        }
    }
}

struct MessageVisitor {
    message: String,
}

impl MessageVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
        }
    }
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }
}

// --- Initialization ---

pub static PROMETHEUS_HANDLE: Lazy<Option<PrometheusHandle>> =
    Lazy::new(|| match PrometheusBuilder::new().install_recorder() {
        Ok(h) => Some(h),
        Err(e) => {
            tracing::warn!("Prometheus recorder not installed: {}", e);
            None
        }
    });

pub fn init_tracing() {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,keyforge_hive=debug,tower_http=info".into());

    let _ = Lazy::force(&PROMETHEUS_HANDLE);

    let use_json = env::var("LOG_FORMAT").unwrap_or_default().to_lowercase() == "json";

    // 1. Console Layer
    let console_layer = if use_json {
        tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .json()
            .boxed()
    } else {
        tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .boxed()
    };

    // 2. Log Capture Layer (In-Memory)
    let capture_layer = LogCaptureLayer;

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(capture_layer);

    // 3. Optional File Layer
    if let Ok(log_dir) = env::var("KEYFORGE_LOG_DIR") {
        let file_appender = tracing_appender::rolling::daily(log_dir, "hive.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        std::mem::forget(guard);

        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .json()
            .with_ansi(false)
            .boxed();

        init_final(registry.with(file_layer));
    } else {
        init_final(registry);
    }
}

fn init_final<S>(subscriber: S)
where
    S: tracing::Subscriber
        + Send
        + Sync
        + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    if env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() {
        match opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()
        {
            Ok(exporter) => {
                let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
                    .with_batch_exporter(exporter)
                    .build();

                use opentelemetry::trace::TracerProvider;
                let tracer = provider.tracer("keyforge-hive");
                let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

                subscriber.with(telemetry_layer).init();
                tracing::info!("🔭 Distributed Tracing Enabled (OTLP)");
            }
            Err(e) => {
                subscriber.init();
                tracing::warn!(
                    "OTLP exporter setup failed; continuing without tracing: {}",
                    e
                );
            }
        }
    } else {
        subscriber.init();
        tracing::info!("📝 Local Logging Enabled");
    }
}

pub fn get_metrics_handle() -> Option<PrometheusHandle> {
    PROMETHEUS_HANDLE.clone()
}
