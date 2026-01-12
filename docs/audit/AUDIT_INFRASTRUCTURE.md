# Audit: Infrastructure & Persistence

## 1. The Repository "God Object"
**Location**: `apps/keyforge-hive/src/infra/repositories/jobs.rs`
**Deficiency**: The `claim_job` method contains a 50-line SQL query using `jsonb_build_object` to manually reconstruct the `JobConfig` DTO.
**Impact**: Extreme fragility. Any change to the `JobConfig` struct requires a simultaneous, complex update to a raw SQL string that lacks compiler type-checking.
**Remediation**: Use SQLX's `query_as!` with a flattened "JobRow" struct and implement a `From<JobRow> for JobConfig` mapper in Rust.

## 2. Incomplete Network Client
**Location**: `libs/keyforge-infra/src/net/client.rs`
**Deficiency**: `HiveClient` returns `reqwest::RequestBuilder` instead of typed responses.
**Impact**: Callers (UI, Agent, CLI) must all manually implement the same `.post("jobs").json(&req).send().await...` boilerplate and path logic.
**Remediation**: Implement high-level methods like `client.submit_job(...)` and `client.get_status(...)`.

## 3. Panic on Initialization
**Location**: `apps/keyforge-hive/src/state.rs` -> `AppState::new`
**Deficiency**: Uses `.expect("Failed to connect...")` for the Valkey coordinator.
**Impact**: The server cannot start if the cache layer is momentarily down or slow to respond during container orchestration (e.g., Docker Compose race).
**Remediation**: Change `AppState::new` to return `Result<Self, AppError>` and implement a retry/reconnect loop for critical infra.

## 4. Migration Confidence Fallback
**Location**: `apps/keyforge-hive/src/infra/repositories/jobs.rs` -> `prune_stale_jobs`
**Deficiency**: Contains a hardcoded fallback for a missing `node_id` column.
**Impact**: Technical debt. It masks failures in the migration system and complicates the codebase with "legacy" paths that should have been migrated.
**Remediation**: Enforce strict migrations. Remove the fallback once the schema is verified.
