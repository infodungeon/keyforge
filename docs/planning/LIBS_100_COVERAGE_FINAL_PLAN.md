# Master Plan: 100% Unit Test Coverage for `libs/*`

**Objective:** Systematic closure of all coverage gaps across the KeyForge libraries.
**Doctrine:** Efficiency through micro-builds. Test only the target module until clean, then reconcile the crate.

---

## 1. keyforge-physics (The Math Kernel)
*Estimated Completion: 98%*

*   [ ] **Task PHY-1.3: Trigram Frequency Overflow**
    *   **File:** `src/engines/intel_comet_lake.rs`
    *   **Action:** Add test `test_trigram_freq_overflow`. Manually set `trigram_freqs` to `u32::MAX` and `penalty_redirect` to a high value to force `i64` overflow in `score_trigrams`.
*   [ ] **Task PHY-1.4: Geometric Boundaries (NaN/Inf)**
    *   **File:** `src/kernel/mechanics.rs`
    *   **Action:** Add test `test_mechanics_invalid_geometry`. Inject `f32::NAN` into `KeyNode` coordinates and verify `calculate_pair_cost` returns `Err(PhysicsError::InvalidInput)`.
*   [ ] **Task PHY-1.5: Static Cost Branch Exhaustion**
    *   **File:** `src/verify.rs`
    *   **Action:** Add test `test_resolve_static_exhaustive`. Explicitly hit the "Unknown" finger and "Outer" zone branches by constructing a key with `FingerIndex::new_unchecked(99)` and `ColIndex(10)`.

---

## 2. keyforge-model (The Data Contract)
*Estimated Completion: 85%*

*   [ ] **Task MOD-1.1: KLE Error Coverage**
    *   **File:** `src/geometry/kle.rs`
    *   **Action:** Test parsing of JSON with missing `keys` array, missing `meta`, and invalid `rotation` types.
*   [ ] **Task MOD-2.2: Search Parameter Boundaries**
    *   **File:** `src/config/search.rs`
    *   **Action:** Implement `test_search_params_validation_boundaries`. Exhaustively test every `if` check in `Validator for SearchParams` (e.g., `temp_min` too low, `opt_limit` mismatch).
*   [ ] **Task MOD-3.1: KeyCode Display/FromStr**
    *   **File:** `src/types.rs`
    *   **Action:** Ensure `Display` and `FromStr` for `KeyCode` are hit for both numeric and (if applicable) string labels.

---

## 3. keyforge-infra (The Adapter Shell)
*Estimated Completion: 70%*

*   [ ] **Task INF-2.1: Valkey Provider (0% Coverage)**
    *   **File:** `src/asset/valkey_provider.rs`
    *   **Action:** Create `mod tests`. Mock `DistributedCoordinator` to verify:
        *   `fetch_blob` success/not_found.
        *   `hydrate_mpk` decompression error (inject bad zstd).
        *   `load` category mapping (ensuring `Rubric` works).
*   [ ] **Task INF-3.1: Corpus Utility Synthesis**
    *   **File:** `src/util/corpus.rs`
    *   **Action:** Add tests for `inject_synthetic_data` and `populate_corpus_from_segments` with empty/malformed segments.
*   [ ] **Task INF-4.1: Atomic IO Errors**
    *   **File:** `src/fs/io.rs`
    *   **Action:** Add `test_atomic_write_fail`. Attempt to write to a read-only directory or a path that is a directory; verify `Err`.

---

## 4. keyforge-evolution (The Search Engine)
*Estimated Completion: 90%*

*   [ ] **Task EVO-1.1: State Saturation**
    *   **File:** `src/supervisor/state.rs`
    *   **Action:** Add `test_score_saturation`. Manually increment `current_score` until it hits `i64::MAX` and verify the `checked_add` fallback logic in `Optimizer::step`.
*   [ ] **Task EVO-2.1: IPS Math Corner Case**
    *   **File:** `src/supervisor/annealing.rs`
    *   **Action:** Add `test_ips_zero_elapsed`. Mock `TimeKeeper` to return `Duration::ZERO` and verify `ips` becomes `0.0` without panicking.

---

## 5. keyforge-export (The Firmware Layer)
*Estimated Completion: 0%*

*   [ ] **Task EXP-1.1: QMK Generation**
    *   **File:** `src/qmk.rs`
    *   **Action:** Add `test_qmk_export_complex`. Export a layout with 3+ layers and verify the C macro structure.
*   [ ] **Task EXP-2.1: ZMK Edge Cases**
    *   **File:** `src/zmk.rs`
    *   **Action:** Test recursion depth limits in ZMK combo generation (if applicable) and long-label sanitization.

---

## 6. Remaining Libraries (Cleanup Phase)

*   [ ] **Task PER-1.1: Persistence Compiler Errors (`keyforge-persistence`)**
    *   Test: `test_compiler_persistence_fail`. Mock a database connection failure during query recording.
*   [ ] **Task PRO-1.1: Serde Size Limits (`keyforge-protocol`)**
    *   Test: `test_serde_limit_error`. Use `deserialize_limited_vec` with a JSON array exceeding `MAX_TRANSPORT_VEC_LEN`.
*   [ ] **Task SEC-1.1: Drop Zeroization (`keyforge-security`)**
    *   Test: Verify `SecretBytes` can be created and moved without issue. (Note: True zeroization verification usually requires heap inspection, but we can verify API surface).
*   [ ] **Task COR-1.1: Newtype Range Invariants (`keyforge-core`)**
    *   Test: Ensure `KeyIndex` and `FingerIndex` construction from `usize` handles truncation/overflow as expected.

---

## 7. Execution Protocol (Efficiency)

1.  **Iterate:** `cargo test -p <crate> --lib <path>::tests::<task_test_name>`
2.  **Crate Cleanup:** Once tasks are checked, run `cargo test -p <crate>`.
3.  **Final Reconciliation:** `just cover <crate>` to confirm 100%.
