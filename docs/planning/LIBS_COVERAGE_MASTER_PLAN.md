# Libs Coverage Master Plan: 100% Unit Test Strategy

**Objective:** Achieve 100% branch and line coverage for all crates in `libs/` using an efficient, batched development cycle.
**Primary Constraint:** Minimize tool-induced idle time (avoiding 3-5 minute `tarpaulin` runs for minor changes).

## 1. Tactical Workflow (The "LID" Loop)

To remain efficient, we decouple implementation from full verification:

1.  **Phase Identification:** Select a target crate from the backlog.
2.  **Coverage Audit:** Run `just cover <crate>` once to generate an XML/HTML report.
3.  **Target Selection:** Identify "Logic Dark Matter" (uncovered branches) in specific files.
4.  **The Fast Loop (Unit Testing):**
    *   Write a unit test targeting the specific uncovered logic.
    *   Verify with `cargo test -p <crate> --lib <module>` (**< 10s**).
    *   Iterate until all known branches in that module are hit.
5.  **The Reconciliation (Batch Verification):**
    *   After a set of modules is "cleared," run `just cover <crate>` to confirm 100%.

---

## 2. Crate Backlog & Complexity Mapping

| Crate | Est. Complexity | Current Status | Key Targets |
| :--- | :--- | :--- | :--- |
| `keyforge-physics` | High (Math/SIMD) | ~95% | SIMD Fallbacks, Overflow Errors |
| `keyforge-model` | Medium (Data/KLE) | ~20% | Parsing, Validation, Weights |
| `keyforge-evolution` | High (State) | ~0% | Optimizer Stepping, Annealing Math |
| `keyforge-compute` | Medium (Pipeline) | ~0% | Scorer Builder, Registry |
| `keyforge-infra` | Medium (IO) | ~0% | FS/Valkey Providers, Config |
| `keyforge-persistence`| Low (Repo) | ~0% | SQLx Mocks (Offline), Repos |
| `keyforge-protocol` | Low (Serde) | ~0% | Error Enums, API Types |
| `keyforge-runner` | Medium (Process) | ~0% | Agent Integration, WAL |
| `keyforge-security` | Medium (Auth) | ~0% | Token Logic, Permissions |
| `keyforge-core` | Low (Utils) | ~0% | Newtypes, Constants |

---

## 3. Detailed Phase Tasks

### Phase 1: The Physics Nucleus
*   **Physics (`keyforge-physics`):**
    *   Hit `ScoreOverflow` paths in `verify.rs` by injecting MAX values into `Corpus`.
    *   Verify `IntelScoringEngine` scalar paths by forcing architecture-independent calls.
    *   Exhaustively test `kernel/compute/analysis.rs` metric detection (all roll/redirect variants).
*   **Evolution (`keyforge-evolution`):**
    *   Implement "Ghost Model" tests for the `Optimizer`.
    *   Test `Annealing` temperature curves for monotonicity and boundary limits.
*   **Compute (`keyforge-compute`):**
    *   Test the `PipelineBuilder` for invalid stage sequences.

### Phase 2: The Semantic Contract
*   **Model (`keyforge-model`):**
    *   **KLE Parsing:** 100% coverage on `geometry/kle.rs` using a library of "Broken" vs "Valid" JSONs.
    *   **Weights:** Test every permutation of `Rubric` and `CostModel` serialization.
*   **Protocol (`keyforge-protocol`):**
    *   Verify every `ForgeError` variant can be serialized/deserialized without loss.

### Phase 3: The Operational Shell
*   **Infra (`keyforge-infra`):**
    *   **FS Provider:** Use ephemeral `tempfile` to test directory recursion and permission errors.
    *   **Caching:** Test LRU eviction and "Stale-while-revalidate" logic.
*   **Persistence (`keyforge-persistence`):**
    *   Test repository traits using in-memory SQLite (if applicable) or strict mock objects.

### Phase 4: Integration & WASM
*   **Security (`keyforge-security`):**
    *   Test token expiration edge cases (T-1s, T+1s).
*   **Wasm (`keyforge-wasm`):**
    *   Test JS-to-Rust type conversion boundaries.

---

## 4. Measurement of Success
*   **Success Condition:** `cargo tarpaulin --workspace` reports **100%** on all lines in `libs/`.
*   **Interim Checkpoints:** XML coverage reports stored in `target/coverage/` for audit trails.
