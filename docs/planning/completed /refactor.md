# Architecture Decision Record: Decoupling Data Models, System Architecture, and Testing Strategy

**Date:** January 17, 2026
**Status:** Proposed
**Context:** The current architecture suffers from high coupling and rigidity across data models (`CostModel`, `SearchParams`) and the testing infrastructure. Modifications trigger cascading recompilations, massive test rework, and confusion regarding system stability.

## 1. The Problem: Data & Schema Rigidity

Currently, volatile configuration data is defined as explicit fields in Rust structs.

**Examples of Rigidity:**
1. **Physics (`CostModel`):** `pub sfb_penalty: f32`. Adding "lateral_stretch" breaks the schema.
2. **Optimization (`SearchParams`):** `pub temp_min: f32`. Adding "genetic_mutation_rate" breaks the schema.

**Consequences:**
1. **High Coupling:** The UI and DB layers must know about specific physics parameters they don't use.
2. **Fragility:** Experimental changes require full stack refactors.

## 2. The Testing Mandate (Requirements)

We acknowledge five critical failures in the current testing approach. The new strategy must satisfy these explicit requirements:

1.  **Rigorous Unit Testing:** All logic, edge cases, and mathematical invariants must be tested in the units (`src/`). "Rigorous" means 95%+ branch coverage and exhaustive property-based testing where applicable.
2.  **Zero Duplication:** Unit logic must **not** be re-tested in the integration layer. Integration tests should assume units are correct and focus solely on the "wiring" (IO, orchestration, and crate boundaries).
3.  **Crate Affinity:** Integration tests must reside in the correct crate (the crate that owns the integration point).
4.  **Documentation:** Tests must be well-documented. Intent and desired result must be obvious.
5.  **Strategy Statement:** We must define a strategy that prioritizes **Maintainability, Extensibility, and Robustness to Change** over raw coverage.

## 3. The Solution: Data-Driven Configuration

We will transition from **Explicit Structs** to a **Parameter Map** pattern for volatile data.

### A. Redefining the Data Models (`keyforge-model`)

```rust
pub struct ScoringWeights {
    // Common fields for backward compatibility
    pub sfb_penalty: f32,
    // Capture all new/experimental weights
    #[serde(flatten)]
    pub dynamic_weights: HashMap<String, f32>,
}
```

### B. The Consumer as Gatekeeper

The consuming crates (`keyforge-physics`, `keyforge-evolution`) become the sole owners of parameter semantics, querying the map for values they recognize.

## 4. The Solution: Restructured Testing Strategy

We will adopt a strict **Testing Hierarchy** to satisfy the Mandate.

### A. Unit Tests (`src/**/mod.rs`) -> Satisfies Mandate #1
*   **Scope:** Pure logic, algorithms, math, and private state transitions.
*   **Location:** Inside the `src/` directory of the defining crate (e.g., `libs/keyforge-physics/src/kernel/compute.rs`).
*   **Constraint:** Must **not** use `std::fs` or complex setups.
*   **Goal:** Exhaustive verification of *logic* (e.g., "Does the SFB penalty calculate correctly given these inputs?").

### B. Integration Tests (`tests/*.rs`) -> Satisfies Mandate #2
*   **Scope:** Public API surface, module wiring, and cross-crate interactions.
*   **De-duplication Rule:** **Strictly Forbidden** to test internal logic already covered by Unit Tests. If a test is checking a math result, it is a Unit Test. If it is checking if a file was read and passed to the engine, it is an Integration Test.
*   **Goal:** Verify *contract* adherence (e.g., "Does the Loader correctly populate the Physics engine?").

### C. Crate Affinity Audit -> Satisfies Mandate #3
*   **Action:** We will audit every integration test file.
*   **Rule:** If a test in `keyforge-cli` tests `keyforge-physics` logic, it must move to `keyforge-physics`.
*   **Rule:** If a test checks the interaction between `hive` and `postgres`, it belongs in `keyforge-infra` or `keyforge-hive`, not `keyforge-core`.

### D. Documentation Standard (The "Why") -> Satisfies Mandate #4
Every non-trivial test must include a doc comment explaining the **Intent**.

```rust
/// Intent: Verify that the annealing supervisor aborts early if the score stagnates.
/// Expected: The optimizer returns the best result found before the step limit.
#[test]
fn test_annealing_stagnation() { ... }
```

### E. Strategic Robustness (Fixtures) -> Satisfies Mandate #5
Instead of constructing complex structs in code (brittle), load "Golden Data" from `tests/fixtures/`.
*   **Action:** Refactor tests to `load_fixture("scenario_a.json")`.
*   **Benefit:** Changing internal struct fields doesn't break compilation of test files (Robustness).
*   **Benefit:** New scenarios can be added by adding JSON files, not writing code (Extensibility).

## 5. Additional Architectural Flaws & Resolutions

### 1. The "Asset Loader" Tight Coupling
**Resolution: Generic Loader Pattern**
`fn load<T: Asset>(&self, id: &str) -> Result<T>;`

### 2. The "UI-Backend" Contract Rigidity
**Resolution: Schema-Driven UI**
Backend exposes a schema; Frontend generates forms dynamically.

## 6. Implementation Strategy (The Five Waves)

### Wave 1: The Cost Model (Data Decoupling)
*   **Goal:** Allow adding new physics weights without breaking the build.
*   **Action:** Refactor `ScoringWeights` to use `HashMap`.
*   **Status:** [x] Complete.

### Wave 2: Test Architecture & Stability (The Great Migration)
*   **Goal:** Restore trust in the test suite and satisfy the Testing Mandate.
*   **Action A (Audit & Move):** Analyze every file in `tests/`. Move logic tests to `src/`. Ensure crate affinity.
*   **Action B (Fixtures):** Convert code-defined test data to JSON fixtures.
*   **Action C (Docs):** Annotate remaining integration tests with *Intent* and *Expected Result*.
*   **Action D (De-duplicate):** Purge logic checks from the `tests/` directory.
*   **Status:** [x] Complete.

### Wave 3: Loader Cleanup
*   **Goal:** Make adding new asset types easy.
*   **Action:** Refactor `AssetLoader` to use the Generic pattern.
*   **Status:** [x] Complete.

### Wave 4: Search Config & UI Flexibility
*   **Goal:** Support multiple optimization algorithms.
*   **Action:** Refactor `SearchParams` to use `HashMap` and implement Schema-Driven UI.
*   **Status:** [x] Complete.

### Wave 5: Compiler Refactor
*   **Goal:** Improve testability of the physics engine.
*   **Action:** Break `Compiler` into a pipeline (`GeometryStage`, `CostStage`) to allow unit testing of compilation steps.
*   **Status:** [x] Complete.

## 7. Benefits

1.  **Robustness:** Tests verify behavior, not implementation details.
2.  **Velocity:** Changing logic requires updating one Unit Test, not 50 Integration Tests.
3.  **Clarity:** Developers understand *what* a test does before fixing it.
4.  **Flexibility:** Data structures and UI can evolve without breaking the world.