# Backlog: Hive Backend (Phase 1 & 2)

## Database (PostgreSQL)
*   [ ] **Create Migration File**: `migrations/20251224000000_v1_foundation.sql`.
*   [ ] **SQL: Users Table**: `id (UUID), username (Text), password_hash (Text), created_at`.
*   [ ] **SQL: API Keys Table**: `hash (Text), user_id (UUID), label (Text), scopes (JSONB)`.
*   [ ] **SQL: Audit Logs**: `id, action, actor_id, target, details, ip, timestamp`.
*   [ ] **SQL: Jobs Update**:
    *   Add `parent_job_id` (Text, Nullable).
    *   Add `owner_id` (UUID, Nullable).
    *   Add `priority` (Integer, Default 10).
    *   Add `is_public` (Boolean, Default false).

## Authentication & Routing
*   [ ] **Refactor `auth.rs`**:
    *   Create `verify_worker_signature` middleware (Ed25519).
    *   Create `verify_user_key` middleware (API Key).
    *   Keep `verify_admin_secret` middleware (HIVE_SECRET).
*   [ ] **Refactor `lib.rs` (Router)**:
    *   Define `worker_router` (Public endpoints: register, results).
    *   Define `user_router` (Protected endpoints: submit, upload).
    *   Define `admin_router` (Protected endpoints: nuke, stats).
    *   Merge routers into main app.

## Job Logic (`src/api/jobs.rs`)
*   [ ] **Update `register_job`**:
    *   Accept `CostMatrixSource::Custom`.
    *   If Custom, hash the content to generate Job ID (do not save file to disk yet).
*   [ ] **Update `claim_job`**:
    *   Implement Priority Queue logic (ORDER BY priority DESC, created_at ASC).
    *   **Lineage Lookup**: Query `results` table for top 5 layouts matching `parent_job_id`.
    *   **Injection**: Populate `parents` vector in the returned `JobConfig`.
    *   **Baseline**: Calculate and inject `baseline_score` (best of parents).

## Asset Management
*   [ ] **Create `src/api/assets.rs`**:
    *   Implement `POST /user/assets`: Accepts file upload, saves to `data/users/{user_id}/`.
    *   Implement `GET /user/assets`: Lists user's custom files.

## Operations
*   [ ] **Update `main.rs`**: Add graceful shutdown signal handling for DB pool.
*   [ ] **Metrics**: Add `kops_total` counter to Prometheus exporter.
