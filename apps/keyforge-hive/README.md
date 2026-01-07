# KeyForge Hive

The KeyForge Hive is the central coordinator for the distributed layout optimization network. It manages jobs, persists results to PostgreSQL, coordinates via Valkey, and provides a REST/WebSocket API for CLI users and Worker Agents.

## Configuration

The Hive prioritizes configuration settings in the following strict order:

1. **Bootstrap File**: `/etc/keyforge/hive.toml` or the path provided via `--bootstrap`.
2. **CLI Arguments**: Explicit flags passed at runtime.
3. **Environment Variables**: OS-level variables (e.g., `DATABASE_URL`).
4. **Defaults**: Internal fallbacks (e.g., directory `.`).

### Bootstrap File

The bootstrap file is used to anchor the Hive to its data directory before any other infrastructure is initialized.

- **Default Location**: `/etc/keyforge/hive.toml`
- **Field Description**:
  - `data_root`: Absolute path to the KeyForge data directory.

Example `hive.toml`:

```toml
data_root = "/var/lib/keyforge/data"
```

### CLI Flags and Environment Variables

| Flag | Environment Variable | Default | Description |
| :--- | :--- | :--- | :--- |
| `--bootstrap` | `KEYFORGE_BOOTSTRAP` | `/etc/keyforge/hive.toml` | Path to bootstrap TOML. |
| `--data` | `KEYFORGE_DATA_DIR` | `.` | Path to KeyForge data root. |
| `--db` | `DATABASE_URL` | `postgres://...` | PostgreSQL connection string. |
| `--port` | `PORT` | `3000` | HTTP API listen port. |
| `--tls-cert` | `TLS_CERT` | - | Path to TLS Certificate (PEM). |
| `--tls-key` | `TLS_KEY` | - | Path to TLS Private Key (PEM). |
| - | `KEYFORGE_VALKEY_URL` | `redis://127.0.0.1:6379` | Valkey connection string. |

## Distributed Coordination (Valkey)

KeyForge uses Valkey (Redis-compatible) as a high-performance coordination layer to prevent database contention during massive scaling events (e.g., 500+ nodes coming online simultaneously).

- **Write Shield**: Node registration checks Valkey for existing hardware profiles before hitting PostgreSQL. This prevents row-level locking on the `hardware_profiles` table during thundering herds.
- **Real-Time Stats**: Cluster throughput (OPS) and active node counts are aggregated in Valkey via telemetry heartbeats.
- **Asset Manifest**: The authoritative hash of system assets is cached in Valkey for rapid worker synchronization.

### Deployment Architectures

#### 1. Docker / Reverse Proxy (Recommended)

In the standard Docker Compose setup, Hive sits behind an Apache or Caddy web server which handles SSL termination.

- **External**: `https://hive.infodungeon.com` (Port 443) -> Apache
- **Internal**: Apache (ProxyPass) -> `http://hive:3000`
- **Configuration**: Do **not** set `TLS_CERT` or `TLS_KEY`. Hive runs in HTTP mode; the proxy handles encryption.

#### 2. Bare Metal / Native TLS

For standalone servers without a proxy:

1. Generate certificates (e.g., via Certbot).
2. Set `TLS_CERT` and `TLS_KEY` environment variables.
3. Hive will automatically switch to HTTPS mode using `rustls`.

### Logging and Observability

The Hive uses `tracing` for high-performance, non-blocking instrumentation.

| Variable | Default | Description |
| :--- | :--- | :--- |
| `LOG_FORMAT` | `text` | Set to `json` for structured console logs. |
| `RUST_LOG` | `info` | Filter level (e.g., `debug`, `keyforge_hive=trace`). |
| `KEYFORGE_LOG_DIR` | - | If set, Hive writes daily-rotating JSON logs to this path. |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | - | Target for OpenTelemetry gRPC tracing data. |

### Advanced Operational Variables

- `HIVE_SECRET`: Master API key for administrative actions and node registration.
- `HIVE_SERVER_KEY`: Unique ID for this instance (Generated if missing).
- `DATABASE_MAX_CONNECTIONS`: Max DB pool size (Default: 100).
- `DATABASE_TIMEOUT_SECONDS`: DB connection timeout (Default: 10).
- `CORS_ALLOWED_ORIGINS`: Comma-separated list (e.g., `https://app.keyforge.com`) or `*`.
- `RATE_LIMIT_PER_SEC`: Global API requests allowed per second (Default: 60).
- `RATE_LIMIT_BURST`: Maximum spike capacity for global rate limit (Default: 100).
- `MAX_JSON_BODY_SIZE`: Max size (bytes) for incoming JSON requests (Default: 1MB).

## Startup Behavior

### SQLx Offline Mode (Docker Builds)

To build the Docker image, you must first generate the SQLx offline data file. This allows the compiler to verify SQL queries without a live database connection.

```bash
# Ensure DB is running
just db-up
# Generate schema cache
cargo sqlx prepare --workspace --database-url postgres://keyforge:forge_password@localhost:5432/keyforge_hive
```

If the resolved data directory does not exist or is inaccessible, the server logs a `FATAL` error and exits immediately. Hive will attempt to initialize the workspace structure (creating `system/` and `user/` subdirectories) if the root exists but is empty.

## Operations

### Management

Use the `Justfile` in the workspace root for common tasks:

- `just db-up`: Start Postgres via Docker.
- `just db-reset`: Wipe the DB and reapply migrations.
- `just serve-prod`: Start Hive against local data.

### Monitoring

The Hive includes a built-in terminal dashboard for real-time monitoring:

```bash
just serve-monitor
```
