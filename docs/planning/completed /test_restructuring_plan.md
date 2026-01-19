# Crate-Specific Test Restructuring Plan (v8)

**Date:** 2026-01-18
**Status:** COMPLETE (Libraries & Apps)

## 1. Principles
1.  **Unit Tests (Logic):** Must live in the same file as the code they test (`mod tests`).
2.  **Integration Tests (Interfaces):** Must live in the **Consumer** crate (`tests/` directory). Drives the public API as a black box.
3.  **No Mocks in Integration:** Real DB, Real Network, Real FS.
4.  **Refactor-First:** Split monolithic files to enable atomic testing.

---

## 2-8. Tier 1-3 Crate Status
- [x] **keyforge-model:** COMPLETE.
- [x] **keyforge-physics:** COMPLETE.
- [x] **keyforge-evolution:** COMPLETE.
- [x] **keyforge-protocol:** COMPLETE.
- [x] **keyforge-infra:** COMPLETE.
- [x] **keyforge-adapter:** COMPLETE.
- [x] **keyforge-persistence:** COMPLETE.

---

## 9-12. Helper & Application Crate Status
- [x] **keyforge-compute:** COMPLETE.
- [x] **keyforge-core:** COMPLETE.
- [x] **keyforge-wasm:** COMPLETE.
- [x] **keyforge-runner / testing / security / export:** COMPLETE.
- [x] **keyforge-agent:** COMPLETE. (Restructured modules, migrated unit tests).
- [x] **keyforge-hive:** COMPLETE. (Added unit tests for validation/auth).
- [x] **keyforge-cli:** COMPLETE. (Added parser unit tests).
- [x] **keyforge-ui:** COMPLETE. (Migrated security unit tests).
- [x] **keyforge-assetmgr:** COMPLETE. (Refactored to lib + unit tests).
- [x] **keyforge-assets:** COMPLETE.

---

## 13. System Integration (tests/system)

**Status: COMPLETE**

- [x] **Task 13.1 (Infrastructure):** Created `keyforge-system-tests` crate and verified `HermeticWorkspace` bootstrap.

- [x] **Task 13.2 (Production Scenarios):** Moved "Physics Truth" tests using production data to `tests/system/physics_scenarios.rs`.

- [x] **Task 13.3 (Orchestration):** Created `tests/system/orchestration_flow.rs` verifying the full compute pipeline.



---



**FINAL STATUS: WORKSPACE RESTRUCTURED AND VERIFIED.**
