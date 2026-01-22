# KeyForge Technical Debt Registry

## 1. Mathematical Integrity (Tier 1)
- [x] **Roll/Redirect Parity**: Minor drift between exact and scalar engines for complex trigrams. (Remediated 2026-01-21)
- [x] **Roll/Redirect Ground Truth**: Physics logic duplicated across `mechanics.rs` and `exact.rs`. (Remediated 2026-01-21)
- [ ] **Ghost Model Evolution**: Stochastic algorithm verification model not yet implemented.

## 2. Dependency Management
- [ ] **Legacy Remnants**: `proc-macro-hack` is present in the workspace. This is deprecated and unnecessary for Rust versions >= 1.45.
- [x] **Version Duplication**: Redundant versions of `base64`, `bitflags`, `getrandom`, `syn`, and `hashbrown` unified in the workspace Cargo.toml. (Remediated 2026-01-21)
- [ ] **Feature Flag Bloat**: Several crates use "full" or broad feature sets which may be increasing compile times and binary sizes.
- [x] **Architectural Leakage**: `keyforge-infra` and `keyforge-persistence` had direct dependencies on Tier 1 `keyforge-physics`. (Remediated 2026-01-21)

## 3. Architectural Doctrine
- [x] **Missing Ghost Code**: `keyforge-physics` now has a `ghost.rs` reference model. (Remediated 2026-01-21)
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
- [ ] **WASM Type Safety**: `keyforge_wasm.js` lacks thorough verification for `Set` and `Map` types passed over the bridge.
- [ ] **UI Memory Management**: `patchEllipsis` in the UI codebase has potential memory leaks related to tooltip lifecycle management.
- [x] **Duplicated Loader Logic**: `InMemoryLoader` moved to `keyforge-core` and shared with `keyforge-wasm`. (Remediated 2026-01-21)

## 5. Metadata & Registry
- [ ] **Keycode Registry**: The mapping between QMK/ZMK keycodes and internal `KeyCode` types is partially manual. A code-generated registry from a schema would reduce errors.
- [x] **Magic Numbers**: Heuristic thresholds in `heuristics.rs` moved to `constants.rs`. (Remediated 2026-01-21)
- [x] **Validation Consistency**: Unified validation strategy implemented in protocol and model layers. (Remediated 2026-01-21)