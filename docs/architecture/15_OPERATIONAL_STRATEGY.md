# Operational Strategy (Day-2 Operations)

**Version:** 4.3
**Context:** Maintenance, Scaling, and Reliability.

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
*   **Function:** `set_manifest_entry`
*   **Role:** Stores the authoritative SHA-256 hash of system assets. This allows workers to verify they have the latest corpus/weights without thrashing the disk I/O on the server.

### D. The Event Bus (Pub/Sub)
*   **Function:** `publish_update`
*   **Role:** Broadcasts job status changes (e.g., "Job 123 Completed") to all connected clients instantly, regardless of which server instance they are connected to.

### Operational Configuration
*   **Connection:** `KEYFORGE_VALKEY_URL` (e.g., `redis://:password@127.0.0.1:6379`).
*   **Eviction:** `maxmemory-policy` set to `allkeys-lru` to ensure old heartbeats die first if RAM is full.
*   **Persistence:** `--save ""` (Ephemeral). We do not persist heartbeats across restarts to avoid "Zombie Nodes".

## 3. Secret Management

* **Storage:** Environment Variables (`HIVE_SECRET`, `DATABASE_URL`, `KEYFORGE_VALKEY_URL`).
* **Rotation:**
  1. Update the Secret in the Orchestrator.
  2. Trigger a Rolling Restart.
  3. Nodes re-authenticate with the new secret.

## 4. Backpressure & Overload

* **Job Queue:** Bounded. If the DB Queue table exceeds N rows, `POST /jobs` returns `503 Service Unavailable`.
* **Concurrency:** `tokio::semaphore` limits the number of concurrent DB connections.
* **Rate Limiting:** `governor` middleware rejects excessive API calls.

## 5. Disaster Recovery

* **RPO:** 24 Hours (Daily Backups).
* **RTO:** 1 Hour (Redeploy Stack).
* **Strategy:** Postgres Dump for persistent state. Assets re-downloaded from upstream.
