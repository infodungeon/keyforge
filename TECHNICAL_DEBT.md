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

## 3. Architectural Doctrine
- [ ] **Missing Ghost Code**: `keyforge-physics` and `keyforge-evolution` do not yet have `ghost.rs` reference models as mandated for Tier 1 logic.
- [ ] **Sentinel Fragility**: The `XXXXXXX` string for `KC_NO` is used as a literal in many locations. While centralized in `keyforge-model`, matching should be done via enum variant or constant reference rather than hardcoded strings.

## 4. Frontend & WASM
- [ ] **WASM Type Safety**: `keyforge_wasm.js` lacks thorough verification for `Set` and `Map` types passed over the bridge.
- [ ] **UI Memory Management**: `patchEllipsis` in the UI codebase has potential memory leaks related to tooltip lifecycle management.

## 5. Metadata & Registry
- [ ] **Keycode Registry**: The mapping between QMK/ZMK keycodes and internal `KeyCode` types is partially manual. A code-generated registry from a schema would reduce errors.
