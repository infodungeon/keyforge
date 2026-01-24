# Technical Debt Backlog (Exhaustive)

This document is the result of the 2026-01-23 Forensic Audit. It tracks all known technical debt across 8 systemic categories.

## 0. Prevention & Engineering Safeguards (The gatekeepers)

- [ ] **Architecture Enforcer Expansion (Safe-1)**: Update `ops/scripts/check_arch.py` to audit internal crate imports. Must fail build if `keyforge-ui` uses `std::sync::Mutex` or if `physics`/`evolution` perform direct IO.
- [ ] **Pattern Sentinel Implementation (Safe-2)**: Create `ops/scripts/pattern_sentinel.sh` to scan for and block forbidden patterns in CI: `.unwrap()`, `.to_string()` in error mapping, and unsupervised `tokio::spawn`.
- [ ] **Dependency Guardrail Policy (Safe-3)**: Configure `cargo deny` to enforce a strict `workspace = true` policy for all shared dependencies to prevent future version fragmentation.
- [ ] **Continuous Parity Oracle (Safe-4)**: Implement a dedicated CI step for "Shadow Execution" that verifies 1,000+ random layouts against both the optimized engine and the `DeterministicScorer`.

## 1. High-Priority Remediation (Safety & Integrity)

- [ ] **Panic Scrub (Code-1)**: Replace ~50 `.unwrap()` calls in `keyforge-physics` with `Result` handling. (GitHub Issue #2)
- [ ] **Verification Dark Matter (Ver-1)**: Implement unit tests for `physics/kernel/compute/` (scoring, delta, flow). (GitHub Issue #12)
- [ ] **Error Restoration (Code-2)**: Eliminate `.to_string()` error erasure in `infra`, `model`, and `tauri-commands`. (GitHub Issue #3)
- [ ] **Build Graph Repair (Build-1)**: Fix duplicate key collision in root `Cargo.toml` and deduplicate `Cargo.lock` (windows-sys, hashbrown). (GitHub Issue #4)
- [ ] **SIMD Activation (Perf-1)**: Implement Intel AVX2 and ARM NEON kernels (currently scalar stubs). (GitHub Issue #7)

- [ ] **Heavyweight Integration Tests (Perf-4)**: Hive integration tests spawn a new Docker container (Valkey/Redis) for every single test case via `testcontainers`. This causes extreme resource contention and timeouts in CI (e.g., `test_api_user_nuke_unauthorized` timeout).
- [ ] **Truth vs. Implementation (Arch-2)**: Decouple the "Engineering Truth" (Bit-Perfect result) from the implementation path (f64 math). Update `00_MANIFESTO.md` and `11_SCORING_LOGIC.md` to reflect that any engine achieving result parity is "Bit-Perfect."
- [ ] **Atomicity Scrub (Data-2)**: Replace unsafe `fs::write` calls in `sync.rs` and `network.rs` with `atomic_write`.
- [ ] **Lock Contention (Perf-3)**: Refactor `std::sync::Mutex` in `keyforge-evolution` hot-loops to use atomics.
- [ ] **Schema Parity (Doc-2)**: Synchronize `architecture/14_CONFIGURATION_SCHEMA.md` with `search.rs`. Docs currently refer to `start_temp`/`end_temp`, but code uses `temp_max`/`temp_min`.
- [ ] **Supervision Debt (Ops-3)**: Replace unsupervised `tokio::spawn` calls with managed `JoinHandle` tracking.

## 2. Structural & Architectural Debt

- [ ] **Dual-Config Redundancy (Arch-5)**: Consolidate `SearchConfig` (Enum) and `SearchParams` (Map) in `keyforge-model`. The current duplication leads to validation and default-value drift.
- [ ] **Contract Discoverability (Arch-6)**: Transition `SearchParams` from a flattened HashMap to an explicit struct or provide a JSON-Schema-based discovery endpoint for dynamic parameters.
- [ ] **Lock Synchronization (Arch-4)**: Standardize on either blocking or async locks in `keyforge-ui`.
- [ ] **Arrow Code Remediation (Code-4)**: Refactor deeply nested logic in `agent/compute.rs` and `calibration.rs` using the Humble Object pattern or early-return guards.
- [ ] **Memory Hack Cleanup (Code-5)**: Investigate and remove `std::mem::forget` in `observability.rs`. Document the locking invariant that required this bypass.
- [ ] **God Object Decomposition (Arch-3)**: Refactor bloated structs (`KeyNode`, `Rubric`, `AnalysisReport`, `JobConfig`) into nested composition.
- [ ] **Layer Purity (Arch-1)**: Move logic leaked into `apps/keyforge-ui/src-tauri/src/commands/` into the appropriate client libraries.
- [ ] **Ghost Oracle Parity (Ver-2)**: Implement parity-check tests for `libs/keyforge-physics/src/ghost.rs`.
- [ ] **Hardware Detection (UX-1)**: Replace mock CPU detection in `keyforge-agent` with `raw-cpuid` and `sysinfo` telemetry.

## 3. Low-Priority / Maintenance Debt

- [ ] **Allocation Scrub (Perf-2)**: Refactor physics hot-loops to use references/borrows instead of `.clone()` for corpus arrays.
- [ ] **Asset Configuration (Data-1)**: Move hardcoded asset filenames from `model/paths.rs` to a configurable registry.
- [ ] **Command Stubs (UX-2)**: Implement `CancelJob` in `keyforge-hive`.
- [ ] **Manifesto Alignment (Doc-1)**: Update `00_MANIFESTO.md` to reflect the current crate tiering and dependency rules.

## 4. Recently Remediated (Historical Log)

*   **[2026-01-23] Error Erasure in WASM**: Rust errors converted to structured `WasmError` DTOs.
*   **[2026-01-23] Infra/Persistence Layer Inversions**: `AssetLoader` moved to `keyforge-model`.
*   **[2026-01-23] UI Category Type Correction**: Aligned TypeScript definitions with Backend data structure.
*   **[2026-01-22] Oracle Performance**: `ExactScoringEngine` now uses high-performance $O(1)$ delta logic.
*   **[2026-01-22] Fat Handler Cleanup**: Extracted node registration logic into `NodeService`.