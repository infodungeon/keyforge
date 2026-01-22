# KeyForge Technical Debt Registry

## 1. Mathematical Integrity (Tier 1)
- [x] **Roll/Redirect Parity**: Minor drift between exact and scalar engines for complex trigrams. (Remediated 2026-01-21)
- [x] **Roll/Redirect Ground Truth**: Physics logic duplicated across `mechanics.rs` and `exact.rs`. (Remediated 2026-01-21)
- [x] **Ghost Model Evolution**: Stochastic algorithm verification model implemented via `ghost_parity.rs` in evolution crate. (Remediated 2026-01-22)

## 2. Dependency Management
- [x] **Feature Flag Bloat**: `libs/keyforge-physics` includes `tokio` (full) in dev-dependencies. (Remediated 2026-01-22)
- [x] **Unused Dependencies**: `libs/keyforge-protocol` lists `anyhow` in Cargo.toml. (Remediated 2026-01-22)
- [x] **Version Duplication**: Redundant versions of `base64`, `bitflags`, `getrandom`, `syn`, and `hashbrown` unified in the workspace Cargo.toml. (Remediated 2026-01-21)
- [x] **Architectural Leakage**: `keyforge-infra` and `keyforge-persistence` had direct dependencies on Tier 1 `keyforge-physics`. (Remediated 2026-01-21)

## 3. Architectural Doctrine
- [x] **Violations**: `libs/keyforge-export` and `libs/keyforge-runner` illegally use `anyhow` for library error handling. (Remediated 2026-01-22)
- [x] **Positional Arguments**: `libs/keyforge-physics` `EngineFactory` uses positional arguments for engine creation. Refactor to `EngineCompilationContext`. (Remediated 2026-01-22)
- [x] **Missing Ghost Code**: `keyforge-physics` now has a `ghost.rs` reference model. (Remediated 2026-01-21)

## 4. Testing
- [x] **Parity**: `libs/keyforge-physics` lacks property tests for the new `optimal_choice` logic. (Remediated 2026-01-22)

## 5. Export
- [x] **Data-Driven**: Keycode generation was manual string formatting. Transitioned to `KeycodeRegistry` lookup. (Remediated 2026-01-22)
- [x] **Missing Ghost Code**: `keyforge-evolution` now has a `ghost.rs` reference model. (Remediated 2026-01-21)
- [x] **Sentinel Fragility**: The `XXXXXXX` string for `KC_NO` refactored to use constants. (Remediated 2026-01-21)
- [x] **Tier 1 Purity**: High-level scoring wrappers moved out of `keyforge-physics` into `keyforge-compute`. (Remediated 2026-01-21)
- [x] **Logic Duplication**: Centralized `calculate_flow_cost` in `mechanics.rs` to unify ground-truth across implementations. (Remediated 2026-01-21)
- [x] **Leaky Traits**: `AssetServerProvider` now uses `InfraResult` to properly surface errors. (Remediated 2026-01-21)
- [x] **Mixed Abstractions (Hive)**: Orchestration logic in `register_job` moved to `JobService` (Command/Service pattern). (Remediated 2026-01-21)
- [x] **Mixed Responsibility (Persistence)**: `UserRepo` in `keyforge-persistence` refactored to handle only I/O; logic moved to compute. (Remediated 2026-01-21)
- [x] **Domain Model Fragmentation**: `Project` struct unified with `Config` aggregate in `keyforge-model`. (Remediated 2026-01-21)
- [x] **Misplaced Logic**: `StreamingProfileBuilder` moved from `infra` to `keyforge-compute`. (Remediated 2026-01-21)

## 4. Frontend & WASM
- [x] **WASM Type Safety**: Type verification for plain objects/arrays added to `worker.ts` to prevent Set/Map injection errors. (Remediated 2026-01-22)
- [x] **UI Memory Management**: Audited `KeyboardMap.tsx` tooltip lifecycle; verified no leaks in local component state. `patchEllipsis` identified as external `mkdocs-material` dependency. (Verified 2026-01-22)
- [x] **Duplicated Loader Logic**: `InMemoryLoader` moved to `keyforge-core` and shared with `keyforge-wasm`. (Remediated 2026-01-21)

## 5. Metadata & Registry
- [x] **Keycode Registry**: Automated registry generation implemented via `ops/repros/generate_keycodes.rs` from QMK documentation. (Remediated 2026-01-22)
- [x] **Magic Numbers**: Heuristic thresholds in `heuristics.rs` moved to `constants.rs`. (Remediated 2026-01-21)
- [x] **Validation Consistency**: Unified validation strategy implemented in protocol and model layers. (Remediated 2026-01-21)

## 6. Discovered 2026-01-22 (High Priority)

### Tier 1: Physics & Evolution (The Nucleus)
- [ ] **Panic-Free Nucleus**: 
    - `libs/keyforge-evolution/src/ghost.rs`: (Remediated 2026-01-22: GhostOptimizer refactored to return Result)
    - `libs/keyforge-physics/src/ghost.rs`: (Remediated 2026-01-22: Scorer refactored to Result)
    - `libs/keyforge-physics/src/error.rs`: Literal `panic!` in `CalculationError` handling (L77).
    - `libs/keyforge-physics/src/kernel/compute/delta.rs`: Multiple `unwrap()` in swap delta logic (L463, L481).
    - `libs/keyforge-physics/src/kernel/stages/geometry.rs`: `unwrap()` in stage execution (L112, L117).
    - `libs/keyforge-physics/src/kernel/stages/costs.rs`: `unwrap()` in key cost resolution (L165).
- [x] **Primitive Obsession**: Magic number `65536` used in `keyforge-physics` scratch buffers and bounds checks. (Remediated 2026-01-22: Centralized via MAX_KEYCODE_SPACE constant)
- [ ] **State Safety**: Pervasive `lock().unwrap()` in `keyforge-evolution/src/supervisor/annealing.rs`. Should use `map_err` to surface poisoned mutexes as `EvolutionError`.

### Tier 2: Model & Protocol (The Contract)
- [x] **Safe Arithmetic**: `libs/keyforge-model/src/types.rs` `Score` type overloads use `expect()`. (Remediated 2026-01-22: Transitioned to Saturating Arithmetic)
- [ ] **Serialization Safety**: `libs/keyforge-protocol/src/job.rs` and `libs/keyforge-model/src/corpus.rs` use `expect()`/`unwrap()` for `serde_json` operations. Must handle as `Result`.
- [ ] **Incomplete Implementation**: `PosMap::from_slice` in `keyforge-physics` is `unimplemented!`.

### Tier 3: Infra, Hive & UI (The Shell)
- [ ] **UI Type Erasure**: Extensive use of `as any` in `keyforge-ui` API/Worker layers.
- [ ] **Generic Error Handling**: `keyforge-hive` uses `anyhow::Error` as a catch-all in `AppError`. Refactor to specific `ForgeError` variants.
- [ ] **Lazy Defaults**: `ValkeyProvider::load_config_asset` returns `T::default()` on failure. Prevents surfacing underlying I/O or auth errors.
- [ ] **Hardcoded Endpoints**: `localhost:3000` hardcoded in `keyforge-ui` (`NetworkBar.tsx`, `SystemContext.tsx`).

### Final Audit 2026-01-22 (Exhaustive Sweep)
- [ ] **Agent Safety (High)**: `apps/keyforge-agent/src/agent/compute.rs` uses `unwrap()` for score conversion.
- [ ] **Stringly-Typed Metadata (Medium)**: `KeyboardMeta` uses raw `String` for `kb_type`. Should be an enum.
- [ ] **Shell Error Leakage (Medium)**: `keyforge-agent` and `keyforge-assets` use `anyhow::anyhow!`.
- [ ] **Surface-Level Validation (Medium)**: `LayoutValidator` only checks key count.
- [ ] **Unit Test Gaps (Low)**: Performance-critical modules lack internal `mod tests`.
- [ ] **Redundant Identity Logic (Low)**: Overlap between agent identity and infra IO.

### Phase 20 Audit: Reuse & Simplification
- [x] **Lookup Fragmentation (High)**: (Remediated 2026-01-22: Centralized in KeycodeRegistry::resolve_token)
- [x] **Serialization Bloat (Medium)**: (Remediated 2026-01-22: Implemented keyforge-model::utils::json safe wrappers)
- [ ] **Fixture Debt (Medium)**: `setup_minimal` copy-pasted across crates. Move to `keyforge-testing`.
- [ ] **Redundant TempDirs (Low)**: Unified using the `HermeticWorkspace` fixture.

### Phase 21 Audit: Maintainability & Change-Impact
- [ ] **Shotgun Surgery Debt (High)**: Adding a physical metric requires changes in 10+ files.
- [ ] **Pipeline Debt (High)**: CI installs tools from source on every run.
- [ ] **Fragile Guardrails (Medium)**: Boundaries enforced via fragile regexes in CI.
- [ ] **Version Lock Debt (Medium)**: Inconsistent locking of workspace dependencies.
- [ ] **Error Propagation Decay (Low)**: Widespread use of `.to_string()` for error conversion.

### Phase 6 Audit: keyforge-protocol
- [x] **ID Collision Risk (High)**: (Remediated 2026-01-22: Job ID now hashes combined fingerprint of all corpora)
- [ ] **Serialization Ambiguity (Medium)**: `JobRequest` uses `#[serde(flatten)]`.
- [ ] **Static Versioning (Medium)**: Hardcoded `MIN_CLIENT_VERSION`.
- [ ] **Weak Sample Validation (Low)**: `BiometricSample` does not verify characters against registry.

### Phase 7 Audit: keyforge-security & testing
- [x] **Cryptographic Debt (High)**: (Remediated 2026-01-22: Added domain separator and length-prefixing)
- [x] **Floating Point drift (Medium)**: (Remediated 2026-01-22: Payload uses i64 scaled score)
- [ ] **Fixture Rot (Medium)**: Hardcoded JSON blobs in `HermeticWorkspace`.
- [ ] **Blocking Test Setup (Low)**: Synchronous `fs` calls in test setup.

### Phase 8 Audit: keyforge-wasm
- [ ] **Conversion Overhead (High)**: Re-parsing large assets on every WASM call.
- [ ] **Validation Drift (Medium)**: Manual `.validate()` calls in WASM vs Native post-load.
- [ ] **Logic Duplication (Medium)**: Engine compilation duplicated from compute builder.
- [ ] **Error Erasure (Low)**: Errors converted to strings, losing context.

### Phase 9 Audit: keyforge-physics (Kernel & Stages)
- [ ] **Drift Risk (High)**: `GeometryStage` uses `sqrt()` on weighted components, differing from mechanics.
- [ ] **Complexity Debt (Medium)**: Hardcoded "inner/outer" zone logic in costs stage.
- [ ] **Magic Number Bloat (Medium)**: (Remediated 2026-01-22: Now use MAX_KEYCODE_SPACE)
- [ ] **Orchestration Bias (Low)**: Scalar engine hardcoded for analysis reports.

### Phase 10 Audit: keyforge-physics (Engines)
- [ ] **Massive Implementation Duplication (High)**: Engines duplicate scalar scoring logic.
- [ ] **Missing SIMD Implementations (High)**: AVX2 and NEON kernels are stubs.
- [ ] **Concurrency Debt (Medium)**: Use of `thread_local!` for scratch space.
- [ ] **Oracle Performance Debt (Low)**: $O(N)$ deltas in Exact engine.

### Phase 11 Audit: keyforge-physics (Analysis)
- [ ] **Standardization Debt (High)**: Hardcoded standard layouts in Fingerprinter.
- [ ] **Heuristic Performance Debt (Medium)**: Re-allocation of scratch in `suggest_swaps`.
- [ ] **Weak Verification (Medium)**: No Oracle Parity for Analysis metrics.
- [ ] **Precision Loss (Low)**: Hardcoded similarity threshold.

### Phase 12 Audit: keyforge-infra
- [ ] **Hash Verification Debt (High)**: `bootstrap_essentials` ignores manifest hashes.
- [ ] **Durability Debt (Medium)**: `atomic_write` lacks parent `fsync`.
- [ ] **Path Jailing Risk (Medium)**: `run_sync` path join vulnerability.
- [ ] **Magic Asset URL (Low)**: Default asset URL points to localhost.

### Phase 13 Audit: keyforge-model
- [ ] **Semantic Obsession (High)**: Stringly-typed schema in `FingerDefinition`.
- [ ] **Post-Load Inconsistency (Medium)**: Missing `post_load` implementations.
- [ ] **Constants Bloat (Low)**: Overgrown `constants.rs`.

### Phase 14 Audit: keyforge-hive (Orchestration & State)
- [ ] **Permissive CORS Debt (High)**: Insecure default origins in Hive.
- [ ] **Fragile Bootstrap (High)**: Ignored asset server binding failure.
- [ ] **God Router Debt (Medium)**: Monolithic router function.
- [ ] **Telemetry Leakage (Low)**: Raw strings for secrets in config.

### Phase 15 Audit: keyforge-hive (Feature Slices)
- [ ] **Abstraction Leak (Medium)**: Manual orchestration in feature handlers.
- [ ] **Ambiguous Nonce Expiration (Low)**: TTL shorter than job length.

### Phase 16 Audit: keyforge-hive (Services)
- [ ] **Verification DOS Risk (High)**: On-the-fly engine compilation.
- [ ] **Hardcoded Nonce TTL (Medium)**: Hardcoded 600s TTL in VerificationService.
- [ ] **Implicit Config Assumptions (Low)**: Fragile cost model fallback.

### Phase 17 Audit: keyforge-hive (Infrastructure)
- [ ] **Transaction Atomicity Debt (Medium)**: Non-atomic stat retrieval.
- [ ] **Data Type Drift (Medium)**: f32 -> f64 casting for DB scores.
- [ ] **Schema Lock Risk (Low)**: Custom fastrand sleep for migrations.

### Phase 18 Audit: keyforge-hive (Job Repository)
- [ ] **SQL Aggregation Debt (High)**: Brittle manual JSON projections in SQL.
- [ ] **Home Row Drift (High)**: Hardcoded home_row in query.
- [ ] **Non-Canonical Identity (Medium)**: Manual fingerprinting in identity.rs.
- [ ] **N+1 Risk (Low)**: Loop-based key insertion.

### Phase 19 Audit: Design Patterns & Anti-Patterns
- [ ] **Fat Handler Anti-Pattern (High)**: Logic embedded in Hive handlers.
- [ ] **Primitive Drift (Medium)**: 4-6 positional arguments in kernels.
- [ ] **Duplicated Validation Logic (Medium)**: Scattered length checks.
- [ ] **Inconsistent Sync/Async (Low)**: Mixed concurrency model.

### Architectural & Structural Debt (Hotspot Audit)
- [x] **Orchestration Triad Redundancy (High)**: (Remediated 2026-01-22: Merged core/runner into compute)
- [ ] **Redundant ACL (Medium)**: Passthrough adapter logic.
- [ ] **Boundary Contamination (Medium)**: Persistence coupled to filesystem.
- [ ] **Unbound Node Registration (Medium)**: Lack of cluster capacity limits.

### Documentation Debt (Aspirational Hallucination)
- [ ] **Aspirational Design (High)**: Typestate/Command pattern claims vs reality.
- [ ] **Contradictory Failure Logic (High)**: Saturating promise vs panicking implementation. (Remediated 2026-01-22: Score now saturates)
- [ ] **Executable Documentation (Medium)**: Missing doc-tests for architecture.
- [ ] **Interface Headers for LID (Medium)**: Missing signature-only docs.
- [ ] **Stale Architecture Map (Medium)**: Outdated runner/compute relationship.
- [ ] **Unified Documentation Versioning (Low)**: Independent doc versions.
- [ ] **Context Struct Registry (Low)**: Missing Parameter Object cheat sheet.
- [ ] **Split Decision Records (Low)**: Monolithic ADR file.
- [ ] **UX Debt (Low)**: Unsupported click events in Mermaid.

### Technology & Dependency Debt (Ecosystem Audit)
- [ ] **Ecosystem Fragmentation (High)**: rand 0.9 vs fastrand.
- [ ] **Security Risk: JWT Version (High)**: Outdated jsonwebtoken.
- [ ] **Transitive Bloat (Medium)**: Swagger UI in production binaries.
- [ ] **Macro Debt (Medium)**: Excessive lint suppression.
- [ ] **Cryptographic Redundancy (Low)**: Multiple encoding libraries.

### Roadmap to Documentation Parity (Aspirational Alignment)
- [ ] **Implement Typestate for Jobs (High)**: Pending -> Running -> Completed types.
- [ ] **Implement Hive Command Pattern (High)**: Intent vs Execution reification.
- [x] **Harden Score Arithmetic (High)**: (Remediated 2026-01-22: Score now uses saturating ops)
- [ ] **Enforce Context Structs (Medium)**: Refactor kernels to use Parameter Objects.
- [x] **Consolidate Orchestration (Medium)**: (Remediated 2026-01-22: Runner and Core merged into Compute)

### Refactoring Roadmap
- [x] **Unified Registry Lookup**: (Remediated 2026-01-22: resolve_token implemented)
- [x] **Consolidate Orchestrators**: (Remediated 2026-01-22: core/runner merged into compute)
- [ ] **Decouple Persistence**: Abstract FileStore trait.
- [ ] **Harden Adapter**: Commit to divergent schema or remove.