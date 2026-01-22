# KeyForge Technical Debt Registry

## 1. High-Level Progress (Waves 1-11)

### Tier 1: The Nucleus (Physics & Evolution)
- [x] **Ghost Model Evolution**: Stochastic algorithm verification model implemented via `ghost_parity.rs`.
- [x] **Panic-Free Nucleus (Partial)**: Refactored `ghost.rs`, `costs.rs`, `delta.rs`, and `geometry.rs` to Result.
- [x] **Primitive Obsession**: Magic number `65536` centralized via `MAX_KEYCODE_SPACE`.
- [x] **Parameter Object Enforcement**: Introduced `EvaluationContext` in physics kernels.
- [x] **Oracle Performance**: `ExactScoringEngine` now uses high-performance $O(1)$ delta logic. (Remediated 2026-01-22)
- [x] **Math Alignment**: Aligned distance math between `GeometryStage` and `mechanics.rs`. (Remediated 2026-01-22)
- [x] **Kernel Unification**: Refactored `intel_comet_lake`, `arm_neon`, and `generic` kernels to use shared logic. (Remediated 2026-01-22)

### Tier 2: The Contract (Model & Protocol)
- [x] **Safe Arithmetic**: `Score` type now uses saturating arithmetic by default.
- [x] **Typestate Pattern**: Job lifecycle managed via `Pending`, `Running`, `Completed` types.
- [x] **ID Collision Fix**: Job ID now hashes all weighted corpora.
- [x] **Cryptographic Hardening**: PASETO V4.Local implemented; fixed-point signatures.
- [x] **Semantic Obsession**: `FingerDefinition` refactored to type-safe `FingerReach`. (Remediated 2026-01-22)

### Tier 3: The Shell (Infra, Hive & Agent)
- [x] **Orchestration Triad Consolidation**: `core` and `runner` merged into `keyforge-compute`.
- [x] **Hive Command Pattern**: Reified business actions into `HiveCommand` dispatcher.
- [x] **Agent Hot-Loop Hardening**: Hot-loop now handles asset load failures without panics.
- [x] **Secure Sync**: Implemented manifest hash verification and path jailing. (Remediated 2026-01-22)
- [x] **Durable Persistence**: `atomic_write` now enforces parent directory `fsync`. (Remediated 2026-01-22)
- [x] **Constants Modularization**: Split `constants.rs` into `limits`, `paths`, and `physics`. (Remediated 2026-01-22)
- [x] **Fat Handler Cleanup**: Extracted node registration logic into `NodeService`. (Remediated 2026-01-22)

---

## 2. Granular Audit Findings (Detailed Registry)

### Phase 1: keyforge-adapter
- [x] **Identity Debt (High)**: `to_domain_keynode`, `to_domain_keyboard`, and `to_domain_corpus_source` refactored to take references and eliminate redundant clones. (Remediated 2026-01-22)
- [x] **Boundary Leakage (Medium)**: Migrated `KeyAction` and `parse_key` from core model to adapter. (Remediated 2026-01-22)
- [x] **Fragile Parsing (Medium)**: `parse_layout_string_strict` uses naive bracket index lookup (`token.find('(')`) which fails on nested or malformed brackets. (Remediated via `KeycodeRegistry::resolve_token` 2026-01-22)

### Phase 2: keyforge-compute
- [x] **Hardware Selection Bias (High)**: Top-level functions `score`, `analyze` bypass optimized kernels. (Note: Refactored to unified template 2026-01-22)
- [ ] **Platform Limitation (Medium)**: `HardwareProbe` only implemented for x86/x86_64. ARM platforms default to `CpuTopology::default()`.
- [ ] **Mathematical Simplism (Medium)**: `StreamingProfileBuilder::build_model` calculates a "Global Baseline" as a simple arithmetic mean of bigram latencies. Vulnerable to outliers.
- [x] **Leaky Dependencies (Low)**: `keyforge-compute` depended on `raw-cpuid` and `keyforge-physics` optimization flags. (Unified in Triad Merge 2026-01-22)

### Phase 3: keyforge-core
- [x] **Loader Logical Debt (High)**: `InMemoryLoader::load_corpus` ignores the blending requirement and only loads the first provided `CorpusSource`. (Remediated 2026-01-22)
- [x] **Structural Duplication (Medium)**: Near identical wrappers in `core/lib.rs` and `compute/lib.rs`. (Unified in Triad Merge 2026-01-22)
- [ ] **Incomplete Type Mapping (Low)**: `InMemoryLoader` uses `TypeId` and `downcast` for cache retrieval (Type Erasure smell).

### Phase 4: keyforge-export
- [x] **Brittle Logic Debt (High)**: `ZmkExporter` uses nested match statements and manual string stripping (`strip_prefix("KC_")`) rather than leveraging `KeyAction`. (Remediated 2026-01-22)
- [x] **Exporter Duplication (Medium)**: Both `QmkExporter` and `ZmkExporter` implement their own registry lookup and name sanitization. (Unified in `KeycodeRegistry` 2026-01-22)
- [ ] **Visualization UX Debt (Medium)**: `viz/physics.rs` uses hardcoded SVG styles (e.g., `#f8f9fa`, `font-size="3"`). Needs `VizTheme`.
- [ ] **Missing Coverage (Low)**: `libs/keyforge-export/src/util.rs` lacks property-based tests for its sanitization logic (`sanitize_c`, `sanitize_zmk`).

### Phase 5: keyforge-persistence
- [x] **Efficiency Debt (High)**: `UserRepo::save_layout` performs a full read-modify-write of the entire `user_layouts.json` file for every single layout update. (Remediated 2026-01-22)
- [x] **Scalability Debt (High)**: `UserRepo::load_stats_store` has a hardcoded limit of 100,000 biometric samples (L125). (Remediated 2026-01-22)
- [ ] **Fragile Checksum (Medium)**: `PersistedSession::calculate_checksum` uses `serde_json::to_vec` on unsorted fields.
- [ ] **Mixed Concerns (Low)**: `compiler.rs` hardcodes specific asset names like `"keycodes"` and `"qwerty"`.

### Phase 6: keyforge-protocol
- [x] **ID Collision Risk (High)**: `JobConfig::id()` hashes only the first corpus in the `corpora` list (L98). (Remediated 2026-01-22: Now uses combined fingerprint)
- [ ] **Serialization Ambiguity (Medium)**: `JobRequest` uses `#[serde(flatten)]` for `JobConfig`, complicating multi-language client development.
- [ ] **Static Versioning (Medium)**: `check_version_compatibility` hardcodes `MIN_CLIENT_VERSION`.
- [ ] **Weak Sample Validation (Low)**: `BiometricSample` validation does not verify if bigram characters are defined in the `KeycodeRegistry`.

### Phase 7: keyforge-security & testing
- [x] **Cryptographic Debt (High)**: `build_payload` hashes `job_id` and `layout` separately but concatenates without domain separation. (Remediated 2026-01-22)
- [x] **Floating Point drift (Medium)**: Signature payload uses `f32` for scores. (Remediated 2026-01-22: Switched to `i64` scaled score)
- [ ] **Fixture Rot (Medium)**: `HermeticWorkspace` hardcodes JSON blobs instead of using model builders.
- [x] **Blocking Test Setup (Low)**: `HermeticWorkspace::new()` performs multiple synchronous `fs::write` and `fs::create_dir_all` calls.

### Phase 8: keyforge-wasm
- [ ] **Conversion Overhead (High)**: `analyze_layout` and `inject_*` methods use `serde_wasm_bindgen::from_value` for large assets on every call.
- [ ] **Validation Drift (Medium)**: `inject_keyboard` and `inject_corpus` manually call `.validate()`, duplicating `post_load` logic.
- [ ] **Logic Duplication (Medium)**: `analyze_layout` duplicates engine compilation logic from `keyforge-compute/src/builder.rs`.
- [ ] **Error Erasure (Low)**: WASM errors are converted to `JsValue` via `.to_string()`, losing structured context.

### Phase 9: keyforge-physics (Kernel & Stages)
- [x] **Drift Risk (High)**: `GeometryStage` implements its own distance calculation (L58) that uses `sqrt()` on weighted components. (Remediated 2026-01-22)
- [ ] **Complexity Debt (Medium)**: `resolve_key_cost` in `costs.rs` uses deeply nested `match` and `if/else` to resolve zones (L105).
- [x] **Magic Number Bloat (Medium)**: `corpus.rs` hardcodes `65537` and `65536`. (Remediated 2026-01-22: Now uses `MAX_KEYCODE_SPACE`)
- [ ] **Orchestration Bias (Low)**: `lib.rs` hardcodes `ScalarScoringEngine` in its `analyze_with_context` wrapper.

### Phase 10: keyforge-physics (Engines)
- [x] **Massive Implementation Duplication (High)**: `intel_comet_lake.rs` and `arm_neon.rs` both duplicate the full scalar scoring logic. (Unified via shared kernels 2026-01-22)
- [ ] **Missing SIMD Implementations (High)**: `score_layout_avx2` (Intel) and the ARM NEON kernel are both `unimplemented!`/fallback to scalar (L150 in Intel, L50 in ARM).
- [ ] **Concurrency Debt (Medium)**: All optimized engines use `thread_local!` for `PhysicsScratch`.
- [x] **Oracle Performance Debt (Low)**: `ExactScoringEngine` re-scores the entire layout twice for deltas ($O(N)$). (Remediated 2026-01-22)

### Phase 11: keyforge-physics (Analysis)
- [ ] **Standardization Debt (High)**: `Fingerprinter` hardcodes standard layouts (Qwerty, Colemak, Dvorak) as static strings (L42).
- [ ] **Heuristic Performance Debt (Medium)**: `suggest_swaps` allocates a new `PhysicsScratch` (L34) and re-populates a `PosMap` from scratch on every call.
- [ ] **Weak Verification (Medium)**: No "Oracle Parity" test for the `AnalysisReport` metric breakdown.
- [ ] **Precision Loss (Low)**: `identify` uses a hardcoded 0.2 similarity threshold (L85).

### Phase 12: keyforge-infra
- [x] **Hash Verification Debt (High)**: `bootstrap_essentials` ignores the `server_hash` from the manifest. (Remediated 2026-01-22)
- [x] **Durability Debt (Medium)**: `atomic_write` in `fs/io.rs` lacks an `fsync` on the parent directory. (Remediated 2026-01-22)
- [x] **Path Jailing Risk (Medium)**: `run_sync` path join vulnerability. (Remediated 2026-01-22)
- [x] **Magic Asset URL (Low)**: `ClientConfig` hardcodes `http://localhost:3001` as the default `asset_url` (L45). (Remediated 2026-01-22)

### Phase 13: keyforge-model
- [x] **Semantic Obsession (High)**: `FingerDefinition` used raw `HashMap<String, HashMap<String, f32>>`. (Remediated via `FingerReach` 2026-01-22)
- [x] **Post-Load Inconsistency (Medium)**: Missing `post_load` implementing for `CostModel` and `Corpus`. (Remediated 2026-01-22)
- [x] **Constants Bloat (Low)**: `constants.rs` overgrown catch-all. (Modularized 2026-01-22)

### Phase 14: keyforge-hive (Orchestration & State)
- [ ] **Permissive CORS Debt (High)**: `create_app` defaults to allowing localhost:5173/1420 if no origins are specified.
- [ ] **Fragile Bootstrap (High)**: `main.rs` (L180) spawns asset server but only logs error on binding failure; Hive continues running.
- [ ] **God Router Debt (Medium)**: `create_app` 150 lines and manually merges dozens of routes.
- [ ] **Telemetry Leakage (Low)**: `AppConfig` and `RateLimitState` do not implement `Zeroize`.

### Phase 15: keyforge-hive (Feature Slices)
- [x] **Abstraction Leak (Medium)**: `submit_result::handle` (L50) manually orchestrated nonce-checking. (Remediated via `ResultService` 2026-01-22)
- [ ] **Ambiguous Nonce Expiration (Low)**: `DEFAULT_SUBMISSION_EXPIRATION_SECS` used for nonce TTL (L73) may be too short.

### Phase 16: keyforge-hive (Services)
- [x] **Verification DOS Risk (High)**: `VerificationService::verify_score` (L110) performed full compilation without limits. (Remediated via Semaphore 2026-01-22)
- [ ] **Hardcoded Nonce TTL (Medium)**: `VerificationService::verify_signature` (L73) hardcodes `600` seconds.
- [ ] **Implicit Config Assumptions (Low)**: `VerificationService::verify_score` (L125) assumes `cost_raw` string format.

### Phase 17: keyforge-hive (Infrastructure)
- [ ] **Transaction Atomicity Debt (Medium)**: `ResultRepository::get_stats` (L105) performs two separate SQL queries.
- [ ] **Data Type Drift (Medium)**: `insert_batch` (L85) casts `f32` scores to `f64` for Postgres; domain uses `i64`.
- [ ] **Schema Lock Risk (Low)**: `try_init_db` (L60) implements custom retry loop for migrations.

### Phase 18: keyforge-hive (Job Repository)
- [ ] **SQL Aggregation Debt (High)**: `CLAIM_JOB_QUERY` and `GET_JOB_CONFIG_QUERY` use deeply nested `jsonb_build_object`. (Note: Deserialization hardened 2026-01-22)
- [x] **Home Row Drift (High)**: `CLAIM_JOB_QUERY` hardcoded `'home_row', 1`. (Remediated 2026-01-22)
- [ ] **Non-Canonical Identity (Medium)**: `calculate_job_identity` uses manual `format!` and `as_bytes` (L18).
- [ ] **N+1 Risk (Low)**: `ensure_keyboard` (L180 in `core.rs`) inserts keys one-by-one in a loop.

### Phase 19: Design Patterns & Anti-Patterns
- [x] **Fat Handler Anti-Pattern (High)**: Business logic extracted from handlers to `NodeService` and `ResultService`. (Remediated 2026-01-22)
- [x] **Primitive Drift (Medium)**: Kernels still passed 4-6 positional arguments. (Remediated via `EvaluationContext` 2026-01-22)
- [ ] **Duplicated Validation Logic (Medium)**: `submit_layout.rs` manually checks string lengths instead of using `Validator`.
- [ ] **Inconsistent Sync/Async (Low)**: `register_node.rs` uses `tokio::spawn` but performs blocking validation in handler.

### Phase 20: Reuse & Simplification
- [x] **Lookup Fragmentation (High)**: Keycode resolution re-implemented in `ZmkExporter`, `QmkExporter`. (Remediated via `resolve_token` 2026-01-22)
- [x] **Serialization Bloat (Medium)**: Manual `serde_json` calls with repetitive `.unwrap()`. (Remediated via `model::utils::json` 2026-01-22)
- [ ] **Fixture Debt (Medium)**: `setup_minimal` copy-pasted across crates.
- [x] **Redundant TempDirs (Low)**: Tests manually calling `tempfile::tempdir().unwrap()`. (Unified via `HermeticWorkspace` 2026-01-22)

### Phase 21: Maintainability & Operations
- [ ] **Shotgun Surgery Debt (High)**: Adding a physical metric requires changes in 10+ files. (Registry partially implemented).
- [x] **Pipeline Debt (High)**: CI installs `just` and `tarpaulin` from source. (Remediated 2026-01-22)
- [ ] **Fragile Guardrails (Medium)**: Architectural boundaries enforced via `rg` (ripgrep) in CI (L60). Should use `cargo-deny`.
- [x] **Technical Debt Sentinel (High)**: Implemented `ops/scripts/check_debt_integrity.sh` and CI gate to prevent "Analysis Erasure." (Remediated 2026-01-22)
- [ ] **Version Lock Debt (Medium)**: Several crates are locked to specific minor versions while others use workspace inheritance.
- [x] **Error Propagation Decay (Low)**: Errors converted to strings via `.to_string()`. (Remediated in persistence/infra 2026-01-22)
