# Operational Strategy (Day-2 Operations)

**Version:** 4.6
**Context:** Maintenance, Scaling, Reliability, and Containerization.

## 1. Database Migrations

* **Tool:** `sqlx-cli`.
* **Policy:** Zero-downtime migrations.
  * Add columns: Safe.
  * Drop columns: **Forbidden** until code usage is removed in previous deployment.
* **Execution:** The Application Entrypoint runs `sqlx::migrate!().run()` on startup. If migration fails, the container crashes (CrashLoopBackOff) to prevent data corruption.

## 2. Distributed Coordination (Valkey)

KeyForge uses **Valkey** (Redis-compatible) as a high-speed coordination layer. It supplies four critical functions that remove load from the primary database:

### A. The Write Shield (Locking)
*   **Function:** `try_reserve_profile_update`
*   **Role:** Prevents "Thundering Herd" attacks on the `hardware_profiles` SQL table. It acts as a distributed lock/cache, ensuring only *one* Hive instance attempts to write a specific CPU profile to disk per 24 hours.

### B. The Heartbeat Aggregator
*   **Function:** `update_heartbeat` / `get_cluster_stats`
*   **Role:** Stores ephemeral node telemetry (IPS, Temperature, RAM) with a 30-second TTL. This removes 99% of `UPDATE nodes SET ...` traffic from the primary database.

### C. The Source of Truth for Assets
*   **Function:** `set_manifest_entry` / `get_bin`
*   **Role:** Stores the authoritative SHA-256 hash of system assets and the binary content itself. This decouples file serving from the API logic.

### D. The Event Bus (Pub/Sub)
*   **Function:** `publish_update`
*   **Role:** Broadcasts job status changes (e.g., "Job 123 Completed") to all connected clients instantly, regardless of which server instance they are connected to.

## 3. The Asset Lifecycle (Data Plane)

The Asset Architecture follows a strict **Writer/Reader separation**.

### The Manager (`keyforge-assetmgr`)
*   **Role:** The Write Master (Daemon).
*   **Privileges:** Read/Write access to Valkey. Read access to Disk.
*   **Lifecycle:** Long-running process.
*   **Behavior:**
    1.  **Boot:** Scans `data/system`, validates every file, and hydrates Valkey.
    2.  **Runtime:** Watches the filesystem using OS-native events (inotify/FSEvents).
    3.  **Update:** When a file changes, it re-validates and re-uploads atomically.
    4.  **Notify:** Publishes `asset_update` events to the cluster so caches can be invalidated.

### The Server (`keyforge-assets`)
*   **Role:** The Read Replica (Gateway).
*   **Privileges:** **Read-Only** access to Valkey. No Disk access required.
*   **Lifecycle:** Stateless request handler.
*   **Behavior:** Serves HTTP requests by streaming data directly from Valkey memory.

## 4. Containerization Strategy

We adhere to the **Minimal Attack Surface** doctrine.

### Rust Services (`hive`, `assets`, `assetmgr`)
*   **Base Image:** `gcr.io/distroless/cc-debian12`
*   **Components:** Contains only the compiled binary, `glibc`, `libssl`, and `libgcc`.
*   **Security:** No shell (`/bin/sh`), no package manager (`apt`), no temp files.
*   **Size:** ~25MB compressed.

### Infrastructure Services
*   **Database:** `postgres:16-alpine`. Standard minimal image based on Alpine Linux.
*   **Web Proxy:** `httpd:alpine`. Minimal Apache build based on Alpine Linux.

## 5. Secret Management

* **Storage:** Environment Variables (`HIVE_SECRET`, `DATABASE_URL`, `KEYFORGE_VALKEY_URL`).
* **Rotation:**
  1. Update the Secret in the Orchestrator.
  2. Trigger a Rolling Restart.
  3. Nodes re-authenticate with the new secret.

## 6. Disaster Recovery

* **RPO:** 24 Hours (Daily Backups).
* **RTO:** 1 Hour (Redeploy Stack).
* **Strategy:** Postgres Dump for persistent state. Assets re-hydrated automatically by `assetmgr` from the mounted volume or container image.
