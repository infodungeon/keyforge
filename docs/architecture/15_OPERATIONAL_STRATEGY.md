# Operational Strategy (Day-2 Operations)

**Version:** 4.2
**Context:** Maintenance, Scaling, and Reliability.

## 1. Database Migrations

* **Tool:** `sqlx-cli`.
* **Policy:** Zero-downtime migrations.
  * Add columns: Safe.
  * Drop columns: **Forbidden** until code usage is removed in previous deployment.
* **Execution:** The Application Entrypoint runs `sqlx::migrate!().run()` on startup. If migration fails, the container crashes (CrashLoopBackOff) to prevent data corruption.

## 2. Distributed Coordination (Valkey)

KeyForge uses **Valkey** (Redis-compatible) as a high-speed coordination layer.

*   **Role:** Ephemeral state management (Heartbeats, Real-time Stats, Asset Manifests).
*   **Persistence:** RDB snapshots are recommended but not strictly required (state is ephemeral).
*   **Configuration:** `KEYFORGE_VALKEY_URL` (default: `redis://127.0.0.1:6379`).
*   **Failure Mode:** If Valkey is unreachable, Hive will fail to start. If it goes down during operation, real-time stats will freeze, but job submission/result persistence (Postgres) will continue.

## 3. Secret Management

* **Storage:** Environment Variables (`HIVE_SECRET`, `DATABASE_URL`, `KEYFORGE_VALKEY_URL`).
* **Rotation:**
  1. Update the Secret in the Orchestrator (e.g., Docker Swarm / K8s).
  2. Trigger a Rolling Restart.
  3. Nodes re-authenticate with the new secret.

> **Note:** In the current implementation for anonymous volunteer compute, `HIVE_SECRET` may be left blank or set to a public default. The functionality is maintained for future private cluster deployments.

## 4. Backpressure & Overload

* **Job Queue:** Bounded. If the DB Queue table exceeds N rows, `POST /jobs` returns `503 Service Unavailable`.
* **Concurrency:** `tokio::semaphore` limits the number of concurrent DB connections.
* **Rate Limiting:** `governor` middleware rejects excessive API calls before they hit the logic layer.

## 5. Disaster Recovery

* **RPO (Recovery Point Objective):** 24 Hours (Daily Backups).
* **RTO (Recovery Time Objective):** 1 Hour (Redeploy Stack).
* **Strategy:**
  * **State:** Postgres Dump.
  * **Assets:** Re-download from upstream or rebuild from source JSON.
