# KeyForge Architecture & Backlog Status

**Date:** December 26, 2025
**Scope:** Distributed Volunteer Computing Grid

## 1. Critical Remediation (Immediate Action)

### A. Persistence Layer (JSON Standardization)

* **Context:** The `CostMatrixSource::Custom` variant currently panics in the Compiler. CSV parsing was deemed too brittle for n-dimensional biomechanics.
* **Requirement:** Remove CSV support entirely. Update `libs/keyforge-persistence/src/compiler.rs` to parse `Custom` strings as JSON (`RawCostData`).
* **Action:**
  * Remove `csv` dependency if added.
  * Implement JSON deserialization logic in `compiler.rs`.
  * Deprecate/Remove CSV generation logic in `keyforge-cli`.

### B. Infrastructure Stubs (Arena generation)

* **Context:** The UI can collect biometric data, but the backend function `generate_cost_matrix_from_stats` in `libs/keyforge-infra` is a stub returning a CSV header string.
* **Requirement:** Implement logic to convert `UserStatsStore` (Character N-Grams) into `RawCostData` (Key ID transitions).
* **Action:**
  * Inject `KeycodeRegistry` into `UserRepo` to allow character-to-KeyID reverse mapping.
  * Implement the math to convert latencies into cost factors.
  * Serialize output as JSON.

### C. Dependency Safety (Recursion Limits)

* **Context:** `libs/keyforge-infra/Cargo.toml` enables `features = ["unbounded_depth"]` for `serde_json`.
* **Risk:** Allows "JSON Bomb" attacks to crash the Hive via Stack Overflow.
* **Action:** Remove `unbounded_depth` feature flag. Rely on default recursion limits (128).

## 2. Distributed System Architecture

### A. Worker Resilience ("The Zombie Problem")

* **Concept:** Search is a "Massively Parallel Random Walk". Time to complete is irrelevant.
* **Current State:** The Hive "Reaper" kills jobs pending > 10 minutes.
* **New Logic:** Jobs should **never** be killed based on wall-clock time.
* **Action:** Update `apps/keyforge-hive/src/cron.rs` to reset jobs *only* if the assigned Worker has failed to send a Heartbeat for > 5 minutes.

### B. Fault Tolerance (Job Admission)

* **Concept:** Prevent "Poison Jobs" (math edge cases) from crashing the volunteer fleet.
* **Action:** Implement **Admission Control** in the Hive `POST /jobs` endpoint.
  * Perform a "Dry Run" (compile engine + score dummy layout) in memory.
  * If the engine panics or errors, reject the Job Request immediately (`400 Bad Request`).

### C. Protocol Versioning

* **Strategy:** Semantic Versioning + Handshake.
* **Requirement:**
  * Workers declare `version` on connect.
  * Hive stores Worker Version.
  * Scheduler filters Jobs: Do not send v2 Jobs (new physics features) to v1 Workers.
  * **Assets:** Implement content negotiation for data files (Keyboards/Corpora) to prevent clients crashing on new schema fields.

## 3. Operations & Distribution

### A. CI/CD (GitHub Actions)

* **Agent Binaries:** Use a Matrix Build strategy to compile `keyforge-agent` for Linux, Windows, and macOS on every tagged release (`v*`).
* **Registry:** Push the `keyforge-agent` Docker image to **GitHub Container Registry (GHCR)** (Public/Free tier) for ease of volunteer onboarding.

### B. Local Development

* **WASM:** The `Justfile` recipe `ui` now depends on `build-wasm`.
* **Secrets:** `HIVE_SECRET` is treated as optional/legacy. The system defaults to public/open mode for the volunteer grid.

## 4. Undetermined / Future Design (TBD)

* **Contribution Priority:** Logic to prioritize jobs for users who contribute compute to the grid.
* **Global Observability:** A public "Leaderboard" or status page to gamify volunteer contributions.
* **Infinite Fragmentation:** Handling the fact that unique user constraints create unique Job IDs, making direct comparison/leaderboards difficult.
* **Pluggable Engines:** Mechanism to update the Search Algorithm (WASM/DLL) independently of the main Agent binary.
* **Headless UX:** Improving the "double-click" experience for non-technical volunteers running the Agent binary (Config file vs Interactive Wizard).
