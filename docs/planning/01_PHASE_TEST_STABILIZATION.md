# Phase 1 Execution Plan: Test Architecture Stabilization

**Goal:** Refactor the testing suite to comply with the "Testing Mandate" (Rigorous Unit Tests, Zero Duplication, Crate Affinity, Documentation, Fixtures).
**Prerequisite:** None.
**Output:** A robust, fast, and documented test suite that allows safe refactoring of Data Models in Phase 2.

## Work Items

### Group A: Physics Crate (Tier 1 - Critical)
*Focus: The Physics engine is pure math. Tests here must be rigorous units, not integration.*

- [x] **Task A.1: Audit & Inventory.** List all tests in `libs/keyforge-physics/tests/`. Categorize them as "Unit Logic" (Move) or "Wiring" (Keep).
- [x] **Task A.2: Unit Migration.** Move logic tests (e.g., scoring math, swap deltas) to `src/` modules. Verify strict compilation constraints (no `std::fs`).
- [x] **Task A.3: De-duplication.** Delete the migrated tests from `tests/`. Ensure zero overlap.
- [x] **Task A.4: Fixture Extraction.** Identify hardcoded structs in remaining integration tests. Extract to `tests/fixtures/physics/*.json`.
- [x] **Task A.5: Documentation.** Add `/// Intent:` doc comments to all remaining integration tests.
- [x] **Checkpoint A:** Run `just test-core`. Verify success. **Commit:** `refactor(physics): enforce testing mandate`.

### Group B: Evolution Crate (Tier 1 - Critical)
*Focus: Algorithms and state transitions. Determinism is key.*

- [x] **Task B.1: Audit & Inventory.** Audit `libs/keyforge-evolution/tests/`.
- [x] **Task B.2: Unit Migration.** Move mutation logic and annealing schedules to `src/`.
- [x] **Task B.3: De-duplication.** Clean up `tests/`.
- [x] **Checkpoint B:** Run `cargo test -p keyforge-evolution`. Verify success. **Commit:** `refactor(evolution): enforce testing mandate`.

### Group C: Infrastructure & CLI (Tier 3 - Shell)
*Focus: Wiring and IO. Tests here should verify the "Humble Object" pattern.*

- [ ] **Task C.1: CLI Test Affinity.** Check `apps/keyforge-cli/tests/`. If a test verifies physics logic, move it to `libs/keyforge-physics/src`.
- [ ] **Task C.2: Infra Wiring.** Ensure `keyforge-infra` tests focus on file/db interaction, not business logic.
- [ ] **Checkpoint C:** Run `just test-cli`. Verify success. **Commit:** `refactor(infra): enforce testing mandate`.

## Verification & Sign-off

- [ ] **Final Regression:** Run full workspace test suite `just test`.
- [ ] **Coverage Check:** Verify unit test coverage is sufficient (aiming for high branch coverage in Tier 1).
- [ ] **Git Push:** Push the stabilized test suite.
