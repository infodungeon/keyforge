# KEYFORGE MASTER PLAN: v1.0 Alpha Readiness

**Status:** Active
**Target:** Day 1 Public Alpha
**Scope:** Full Stack (Core, Hive, Agent, UI, Ops)

## Legend

| Status | Description |
| :---: | --- |
| ✅ | Completed |
| 🔴 | **Blocked / Critical** (Must fix for Day 1) |
| 🟡 | **In Progress** |
| ⚪ | **Pending** |

---

## PHASE 1: THE IRON FOUNDATION (Architecture & Stability)

**Goal:** Decouple the Core from the OS, secure the API, and stabilize data persistence.

### 1.1 Core Purification (WASM Prep)

* [x] **Refactor `keyforge-model`**
  * [x] Remove `csv` dependency from `Cargo.toml`.
  * [x] Remove `AssetLoader` trait from `src/loader.rs`.
  * [x] Remove `RawCostData` struct (move to infra/workspace).
  * [x] Verify compilation with `--target wasm32-unknown-unknown`.

* [x] **Update `keyforge-infra`**
  * [x] Port `RawCostData` struct (located in workspace loader).
  * [x] Port `AssetLoader` trait definition.
  * [x] Refactor `FsProvider` to implement the new local `AssetLoader` trait.
  * [x] Create `HttpAssetLoader` struct (Implemented as `InMemoryLoader` in `keyforge-wasm`).

* [x] **Update `keyforge-workspace`**
  * [x] Fix imports to point to new `keyforge-infra` locations.

### 1.2 Session Deconstruction

* [x] **Split `Session` Struct**
  * [x] Extract `Engine` logic (Pure Physics) to `src/engine.rs`.
  * [x] Extract `WorkspaceState` (In-Memory Data) to `src/state.rs`.
  * [x] Make `Repository` (Disk I/O) an optional trait bound.

* [x] **Harden Persistence**
  * [x] Refactor `autosave.rs` to use `atomic_write` (temp file + rename).
  * [x] Add error handling for cross-device link errors (fallback to copy+delete).

### 1.3 Database Schema (The Data Contract)

* [x] **Create Migration `20251224000000_v1_foundation.sql`**
  * [x] Create `users` table:
    * [x] `id` (UUID, PK)
    * [x] `username` (Text, Unique)
    * [x] `created_at` (Timestamp)
    * [x] `quota_limits` (JSONB)
  * [x] Create `api_keys` table:
    * [x] `hash` (Text, PK)
    * [x] `user_id` (UUID, FK)
    * [x] `label` (Text)
    * [x] `scopes` (JSONB)
  * [x] Create `audit_logs` table:
    * [x] `id`, `action`, `actor_id`, `target`, `details`, `ip`, `timestamp`.
  * [x] Alter `jobs` table:
    * [x] Add `parent_job_id` (Text, Nullable).
    * [x] Add `owner_id` (UUID, Nullable).
    * [x] Add `priority` (Integer, Default 10).
    * [x] Add `is_public` (Boolean, Default false).

### 1.4 Protocol Upgrade (The API Contract)

* [x] **Refactor `JobRequest` Struct**
  * [x] Change `cost_matrix` field to `CostMatrixSource` enum.
  * [x] Add `parent_job_id: Option<String>`.
  * [x] Add `baseline_score: Option<f32>`.
  * [x] Add `parents: Vec<String>` (for diversity injection).

* [x] **Define `CostMatrixSource` Enum**
  * [x] Variant `Predefined(String)` (Filename on server).
  * [x] Variant `Custom(String)` (Raw CSV content).

### 1.5 Hive Logic (The Brain)

* [x] **Router Refactor**
  * [x] Define `worker_routes` (Public, Ed25519 Signed).
  * [x] Define `admin_routes` (Private, `HIVE_SECRET`).
  * [x] Define `user_routes` (Public, API Key).
  * [x] Merge routers in `lib.rs`.

* [🟡] **Smart Queue Implementation**
  * [x] Update `claim_job` query to use Priority Queue logic.
  * [ ] Implement **Lineage Lookup**: Query `results` for top 5 layouts matching `parent_job_id`.
  * [ ] Implement **Injection**: Populate `parents` vector in the returned `JobConfig`.
  * [ ] Implement **Baseline**: Calculate and inject `baseline_score`.

* [x] **Fix DB Isolation**
  * [x] Ensure `FOR UPDATE SKIP LOCKED` uses `REPEATABLE READ` isolation.

---

## PHASE 2: THE UNIVERSAL GRID (Compute Strategy)

**Goal:** Enable any device (Desktop or Browser) to contribute efficiently.

### 2.1 WASM Core

* [x] **Compilation Config**
  * [x] Update `keyforge-evolution/Cargo.toml` to support `crate-type = ["cdylib"]`.
  * [ ] Create `scripts/build_wasm.sh` (runs `wasm-pack build --target web`).

* [x] **WASM Bindings**
  * [x] Create `struct WasmOptimizer` wrapping the core logic.
  * [x] Implement `step_for_duration(ms: u32)` method.
  * [x] Implement `get_best_layout()` method.

### 2.2 The Web Worker Bridge

* [🔴] **Worker Implementation**
  * [ ] Create `keyforge-ui/src/workers/optimizer.worker.ts`.
  * [ ] Import WASM module.
  * [ ] Handle `onmessage` (Job Payload).
  * [ ] Run optimization loop.
  * [ ] Post `progress` events back to main thread.
  * [ ] Post `result` event when time-box expires.

### 2.3 Adaptive Scheduling (Time-Boxing)

* [🟡] **Agent Refactor (Native)**
  * [x] Modify `compute.rs` to replace `max_steps` loop with `Instant::now() + duration` loop (Implemented as timeout).
  * [ ] Default duration: 60 seconds (configurable via CLI).

* [🟡] **Hive Update**
  * [x] Update `claim_job` to track "Checked Out" timestamp.
  * [ ] Implement Dead Letter Queue logic for jobs that timeout.

### 2.4 Resource Governor

* [🟡] **UI Controls**
  * [ ] Add "Intensity Slider" to Settings View (Low/Med/High).

* [x] **Logic Implementation**
  * [x] **Desktop**: Map slider to `rayon::ThreadPoolBuilder` num_threads.
  * [ ] **Web**: Map slider to number of Web Workers spawned.

### 2.5 Search Logic Updates

* [🔴] **Diversity Pick**
  * [ ] Agent reads `parents` vector from Job Config.
  * [ ] Agent randomly selects one layout string to start.

* [🔴] **Silence Protocol**
  * [ ] Agent reads `baseline_score`.
  * [ ] Logic: `if result.score < baseline_score { submit } else { heartbeat }`.

* [ ] **Heartbeat Signal**
  * [ ] Implement lightweight "Job Done" signal for Hive.

---

## PHASE 3: THE PUBLIC PLATFORM (Web & SaaS)

**Goal:** A public destination for the community to view, share, and submit.

### 3.1 UI Architecture Refactor

* [🔴] **Backend Adapter Pattern**
  * [ ] Create `src/api/backend.interface.ts`.
  * [ ] Create `src/api/tauri.ts` (wraps `@tauri-apps/api`).
  * [ ] Create `src/api/web.ts` (wraps `fetch`).
  * [ ] Create `src/context/BackendContext.tsx` to provide the correct adapter.

* [🔴] **Refactor Components**
  * [ ] Replace all direct `invoke()` calls with `useBackend()` calls.

### 3.2 Virtual Workspace (Web)

* [ ] **Storage Adapter**
  * [ ] Implement `IndexedDB` wrapper for saving layouts/corpora in the browser.
  * [ ] Ensure `SessionContext` loads from IDB when in Web Mode.

### 3.3 Authentication UI

* [🔴] **Views**
  * [ ] Create `src/views/LoginView.tsx`.
  * [ ] Create `src/views/RegisterView.tsx`.
  * [ ] Update `NavRail` to include User Profile.

* [ ] **Logic**
  * [ ] Implement `localStorage` handling for JWT/API Key.

### 3.4 Asset Management

* [🟡] **Custom Uploads**
  * [ ] Implement `POST /user/assets` in Hive.
  * [x] Update UI to upload custom matrices before submitting jobs (`CorpusManager` exists).

### 3.5 Layout Explorer

* [ ] **Public API**
  * [ ] Add `GET /public/layouts` to Hive (Cached).
* [ ] **Frontend**
  * [ ] Create Explorer View (Leaderboard).
  * [ ] Create Detail View (Read-only analysis).

---

## PHASE 4: OPERATIONS & DISTRIBUTION

**Goal:** Professional delivery pipeline and infrastructure.

### 4.1 CI/CD Pipeline (GitHub Actions)

* [x] **Create `.github/workflows/ci.yml`**
  * [x] Trigger on tag push (`v*`).
  * [x] **Job: Build Core**: Run tests.
  * [x] **Job: Build Hive**: Build Docker image, push to registry.
  * [x] **Job: Build Web**: Run `npm build`, upload artifact.
  * [x] **Job: Build Desktop**: Matrix build (Ubuntu, Windows, macOS).

### 4.2 Update Infrastructure

* [ ] **Key Generation**
  * [ ] Create `scripts/gen_keys.sh` (wrapper for `tauri signer generate`).
* [ ] **Manifest Generation**
  * [ ] Create `scripts/gen_update_manifest.js` to scan GitHub Releases and output `update.json`.
* [ ] **Hosting**
  * [ ] Configure Apache container to serve `update.json` with CORS headers.

### 4.3 Database Operations

* [ ] **Backup Service**
  * [ ] Update `docker-compose.yml` to add `backup` service.
  * [ ] Script: `pg_dump` to `/backups` volume every 6 hours.
* [ ] **Restore Guide**
  * [ ] Create `scripts/restore_db.sh`.

### 4.4 Documentation Portal

* [x] **Infrastructure**
  * [x] Initialize MkDocs with Material theme.
  * [x] Configure Docker container for Docs.

* [ ] **Content Creation**
  * [ ] **User**: "Getting Started", "Glossary", "How to Contribute Compute".
  * [ ] **Admin**: "Hosting Hive", "Database Ops".
  * [ ] **Dev**: "Architecture Overview", "API Reference".

Security .. ensure  the mounted data directory has the appropriate external access controls in production.
