# KeyForge Technical Debt Registry

This document tracks identified technical debt, architectural shortcuts, and legacy remnants within the KeyForge workspace. Adherence to the **Engineering Manifesto** requires proactive remediation of these items.

## 1. Build & Verification System
- [x] **Justfile Stale Recipes**: `verify-parity` was pointing to non-existent unit tests. (Remediated 2026-01-21)
- [x] **Broken Integration Tests**: `engine_integration.rs` was broken due to `EngineCapabilities` struct refactor. (Remediated 2026-01-21)
- [ ] **CI/CD Parity**: Ensure `just test-all` is the authoritative source for CI gatekeeping to prevent local vs remote drift.

## 2. Dependency Management
- [ ] **Legacy Remnants**: `proc-macro-hack` is present in the workspace. This is deprecated and unnecessary for Rust versions >= 1.45.
- [ ] **Version Duplication**: The workspace uses multiple versions of core crates (e.g., `reqwest` 0.12 and 0.13). Unify these in the root `Cargo.toml`.
- [ ] **Feature Flag Bloat**: Several crates use "full" or broad feature sets which may be increasing compile times and binary sizes.
- [x] **Architectural Leakage**: `keyforge-infra` and `keyforge-persistence` had direct dependencies on Tier 1 `keyforge-physics`. (Remediated 2026-01-21)

## 3. Architectural Doctrine
- [x] **Missing Ghost Code**: `keyforge-physics` now has a `ghost.rs` reference model. (Remediated 2026-01-21)
- [ ] **Missing Ghost Code**: `keyforge-evolution` still requires a `ghost.rs` reference model for its stochastic algorithms.
- [x] **Sentinel Fragility**: The `XXXXXXX` string for `KC_NO` refactored to use constants. (Remediated 2026-01-21)
- [x] **Tier 1 Purity**: High-level scoring wrappers moved out of `keyforge-physics` into `keyforge-compute`. (Remediated 2026-01-21)
- [x] **Logic Duplication**: Centralized `calculate_flow_cost` in `mechanics.rs` to unify ground-truth across implementations. (Remediated 2026-01-21)

## 4. Frontend & WASM
- [ ] **WASM Type Safety**: `keyforge_wasm.js` lacks thorough verification for `Set` and `Map` types passed over the bridge.
- [ ] **UI Memory Management**: `patchEllipsis` in the UI codebase has potential memory leaks related to tooltip lifecycle management.

## 5. Metadata & Registry
- [ ] **Keycode Registry**: The mapping between QMK/ZMK keycodes and internal `KeyCode` types is partially manual. A code-generated registry from a schema would reduce errors.
- [x] **Magic Numbers**: Heuristic thresholds in `heuristics.rs` moved to `constants.rs`. (Remediated 2026-01-21)
