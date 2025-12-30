# Observability & Telemetry

**Version:** 4.1
**Context:** Logging, Metrics, and Tracing standards.

## 1. Structured Logging (Tracing)

We use the `tracing` crate to emit structured events.

### Log Levels

| Level | Usage | Example |
| :--- | :--- | :--- |
| **ERROR** | Operator intervention required. Data loss or Service outage. | `Database connection failed`, `Worker panic`. |
| **WARN** | Recoverable issues or degraded performance. | `Rate limit exceeded`, `Job retrying`. |
| **INFO** | Lifecycle events. | `Server started`, `Job #123 completed`. |
| **DEBUG** | State transitions and payload details. | `User authenticated`, `Config loaded`. |
| **TRACE** | Hot-path debugging. **DISABLE IN PROD.** | `Score calculated: 10.5`, `Mutex acquired`. |

### Spans

Every request and background job must be wrapped in a Span.

* `http_request`: `{ method, path, request_id, user_id }`
* `job_execution`: `{ job_id, worker_id, attempt }`
* `annealing_loop`: `{ steps, start_temp }`

## 2. Metrics (Prometheus)

We expose a `/metrics` endpoint for scraping.

### Key Indicators (SLIs)

| Metric Name | Type | Description |
| :--- | :--- | :--- |
| `keyforge_jobs_active` | Gauge | Number of jobs currently processing. |
| `keyforge_jobs_total` | Counter | Total jobs submitted (labeled by status: `success`, `failed`). |
| `keyforge_score_latency_ns` | Histogram | Time taken to calculate a single layout score. |
| `keyforge_worker_heartbeats` | Counter | Liveness check for connected agents. |

## 3. Health Checks

* **Liveness:** `/health/live` - Returns 200 if the HTTP server is up.
* **Readiness:** `/health/ready` - Returns 200 if DB is connected and migrations are applied.

## 4. Admin TUI (The Control Plane)

For real-time cluster management without a web browser, we provide a Terminal User Interface.

* **Tool:** `ratatui` (Rust).
* **Binary:** `keyforge-admin`.
* **Connectivity:** Connects via SSH tunnel or local socket to the Hive API.

### Views

1.  **Cluster Map:** Visualizes active Worker Nodes, their CPU load, and current Job ID.
2.  **Job Queue:** Scrollable list of Pending/Running/Failed jobs with "Kill" and "Retry" actions.
3.  **Log Stream:** Tailing `tracing` logs with regex filtering.
