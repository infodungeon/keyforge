# Technical Debt & Deficiency Roadmap

**Status:** Scanning Complete | **Execution Mode:** Systematic Cleanup

This document serves as the authoritative backlog for technical debt. Tasks are grouped by logical changes and prioritized by their impact on system correctness and safety.

---

## 🛠 Active Task List

### 🧱 Wave 1: Core Nucleus & Correctness (Foundation)
*Focus: Ensuring user intent actually reaches the engine and math remains deterministic.*

#### [LOGIC-01] Engine Config Wiring
- [x] **Task [task-persist-001]:** Update `libs/keyforge-persistence/src/compiler.rs` to map `_config` fields (Rubric, SearchParams) into `EngineRequest`.
- [x] **Task [task-wasm-026]:** Update `libs/keyforge-wasm/src/lib.rs` to accept `Rubric` in `analyze_layout`.
- [x] **Task [task-phys-022]:** Standardize `Score` math in `libs/keyforge-model/src/types.rs` to use `saturating_add/sub` globally.
- [ ] **Verification:** Run `just repro repro_score` with custom weights to confirm they are honored.

#### [LOGIC-02] Biometrics & Cost Model Generation
- [x] **Task [task-ui-024]:** Implement `BiometricProfiler` in `libs/keyforge-compute/src/builder.rs` to aggregate `BiometricSample` into `CostModel`.
- [x] **Task [task-ui-024-UI]:** Update `apps/keyforge-ui/src-tauri/src/commands/analysis.rs` to pass biometrics from frontend to builder.
- [ ] **Verification:** Confirm local search results change when a "High Latency" profile is simulated.

#### [SECURITY-01] Cryptographic & IO Hardening
- [x] **Task [task-sec-027]:** Zeroize intermediate buffers in `libs/keyforge-security/src/lib.rs` using `hex::decode_to_slice`.
- [x] **Task [task-sec-029]:** Change `build_payload` in `keyforge-security` to use fixed-point `i64` instead of `f32` bits for signatures.
- [x] **Task [task-infra-008]:** Refactor `libs/keyforge-infra/src/asset/fs_provider.rs` to run `safe_join` before any existence checks.
- [ ] **Verification:** Run `cargo test -p keyforge-security`.

### ⚡ Wave 2: Hot-Path Performance (Optimization)
*Focus: Removing bottlenecks in the Simulated Annealing loop.*

#### [PERF-01] Scoring Kernel Efficiency
- [x] **Task [task-phys-011]:** Move sorted `used_keys` to `EngineContext` in `libs/keyforge-physics/src/kernel/compute.rs`.
- [x] **Task [task-phys-015]:** Implement flow cost memoization or grouped property lookups for trigram $O(C^3)$ reduction.
- [ ] **Verification:** `cargo bench -p keyforge-physics` (Target: 15% reduction in cycle count).

#### [PERF-02] Evolution Loop Cleanup
- [x] **Task [task-evo-017]:** Implement in-place mutation/reversion for 3-way swaps in `libs/keyforge-evolution/src/supervisor/strategies.rs`.
- [ ] **Verification:** `cargo test -p keyforge-evolution` ensures no logic regressions in annealing.

### 🧹 Wave 3: Maintainability & Quality (Polish)
*Focus: Removing magic strings and improving UX/CLI consistency.*

#### [QLTY-01] Code Standardization
- [x] **Task [task-model-018]:** Move all hardcoded constants ("XXXXXXX", model keys, limits) to `libs/keyforge-model/src/constants.rs`.
- [x] **Task [task-infra-020]:** Replace `strip_suffix(".json")` with proper `Path` extensions in `FsProvider`.
- [x] **Task [task-model-025]:** Implement dynamic hand detection in `kle.rs` using X-coordinate clustering.
- [x] **Task [task-export-019]:** Refactor QMK exporter to support recursive AST traversal.

#### [UX-01] Visibility & Reliability
- [x] **Task [task-cli-028]:** Add `indicatif` progress bar to CLI `Search` command.
- [ ] **Task [task-agent-021]:** Implement UUID fallback for machine ID in `keyforge-agent`.
- [ ] **Task [task-ui-002]:** Finalize regex parser for `pinned_keys` in the React frontend.

---

## 📋 Full Deficiency Catalog (Reference)

| ID | Title | Priority | Status |
|---|---|---|---|
| task-persist-001 | Persistence Compiler Ignores Config | 🔴 Critical | ✅ Done |
| task-wasm-026 | WASM Engine Ignores Rubric | 🔴 Critical | ✅ Done |
| task-ui-002 | Pinned Keys Serialization Gap | 🟠 High | ⏳ Pending |
| task-ui-024 | Biometrics Ignored in Local Search | 🟠 High | ✅ Done |
| task-sec-027 | Insecure Secret Key Handling | 🟠 High | ✅ Done |
| task-sec-029 | Non-Deterministic Signature Payload | 🟠 High | ✅ Done |
| task-infra-008 | Path Traversal Risk in FsProvider | 🟠 High | ✅ Done |
| task-infra-020 | Proper Path Extension Handling | 🟠 High | ✅ Done |
| task-phys-011 | Hot-Path Allocation/Sort in PosMap | 🟡 Medium | ✅ Done |
| task-phys-015 | Trigram Triple Nest Optimization | 🟡 Medium | ✅ Done |
| task-phys-022 | Standardize Score Saturation | 🟡 Medium | ✅ Done |
| task-evo-017 | Full Layout Clone on 3-Way Swap | 🟡 Medium | ⏳ Pending |
| task-model-018 | Centralize Magic Values | 🔵 Low | ✅ Done |
| task-model-025 | Dynamic KLE Hand Detection | 🔵 Low | ✅ Done |
| task-export-019 | Recursive ModTap Exporter Support | 🔵 Low | ✅ Done |
| task-cli-028 | CLI Progress Visibility | 🔵 Low | ⏳ Pending |
| task-agent-021 | Machine ID UUID Fallback | 🔵 Low | ⏳ Pending |