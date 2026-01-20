# Libs 100% Coverage: Final Exhaustive Task List

**Objective:** Total coverage for all logic in `libs/`.
**Strategy:** Targeted branch exhaustion and error-path injection.

---

## 1. keyforge-physics (Status: ~95%)
*   [ ] **Task PHY-1: Trigram Loop Exhaustion (`src/verify.rs`)**
    *   **Gap:** Trigram accumulation branches (checked_add/checked_mul).
    *   **Action:** Add `test_oracle_trigram_overflow_checked`. Inject `Score(i64::MAX)` into `penalty_redirect` and frequency `u32::MAX`.
*   [ ] **Task PHY-2: Hardware Detection Fallback (`src/engines/intel_comet_lake.rs`)**
    *   **Gap:** The `#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]` branch.
    *   **Action:** While we cannot easily change the physical CPU, we will wrap the logic in a private function `score_layout_internal` that takes a `bool` for `force_scalar` and test both paths.
*   [ ] **Task PHY-3: Geometric Overflow (`src/kernel/mechanics.rs`)**
    *   **Gap:** `dist_raw` validation for `NaN` and `Infinity`.
    *   **Action:** Add `test_calculate_pair_cost_geometric_fail`. Construct a `Keyboard` with keys at `f32::INFINITY`.

---

## 2. keyforge-model (Status: ~85%)
*   [ ] **Task MOD-1: Configuration Constraints (`src/config/constraints.rs`)**
    *   **Gap:** Currently 0/16 coverage.
    *   **Action:** Implement `test_key_constraint_lifecycle`. Test `FromStr` parsing for constraints (e.g., "0:KC_A") and `validate()` bounds checking.
*   [ ] **Task MOD-2: Source Validation (`src/config/source.rs`)**
    *   **Gap:** Currently 0/28 coverage.
    *   **Action:** Implement `test_corpus_source_serde`. Round-trip `CorpusSource` with `Option<String>` hash and verify `validate()` catches empty IDs.
*   [ ] **Task MOD-3: Keycode Definition Display (`src/keycodes.rs`)**
    *   **Gap:** Display implementation for `KeycodeDefinition`.
    *   **Action:** Add `test_keycode_definition_display`. Assert format matches `"ID: CODE (LABEL)"`.
*   [ ] **Task MOD-4: Layout Logic (`src/layout.rs`)**
    *   **Gap:** Validation logic for `try_from`.
    *   **Action:** Add `test_layout_validation_edge_cases`. Test empty layouts and layouts with 200+ keys.

---

## 3. keyforge-infra (Status: ~85%)
*   [ ] **Task INF-1: Valkey Categorization (`src/asset/valkey_provider.rs`)**
    *   **Gap:** The new `AssetCategory::Rubric` match arm.
    *   **Action:** Implement `test_valkey_provider_rubric_load`. Mock the coordinator to return bytes for `rubrics/test.mpk.zst`.
*   [ ] **Task INF-2: Atomic File Errors (`src/fs/io.rs`)**
    *   **Gap:** `fs::rename` and `File::create` error branches.
    *   **Action:** Add `test_atomic_write_readonly`. Attempt to write to a root-owned or non-existent parent directory.
*   [ ] **Task INF-3: Common Utils (`src/util/common.rs`)**
    *   **Gap:** `calculate_file_hash` error branches (missing file).
    *   **Action:** Add `test_calculate_hash_missing`. Verify it returns `Err`.

---

## 4. keyforge-evolution (Status: ~90%)
*   [ ] **Task EVO-1: Temperature Reset Math (`src/supervisor/state.rs`)**
    *   **Gap:** `reheat_from_best` branch for `start_temp == 0`.
    *   **Action:** Add `test_state_reheat_zero_temp`. Verify it handles zero gracefully.
*   [ ] **Task EVO-2: Performance Metering (`src/supervisor/annealing.rs`)**
    *   **Gap:** `ips` calculation when `elapsed` is extremely small.
    *   **Action:** Add `test_ips_underflow`. Force `elapsed` to `1e-10` and verify result is finite.

---

## 5. keyforge-persistence (Status: ~90%)
*   [ ] **Task PER-1: User Repo Migration Fallback (`src/repo/user_repo.rs`)**
    *   **Gap:** `load_layout_store` when directory exists but file is missing.
    *   **Action:** Add `test_repo_dir_no_file`. Verify it returns an empty store.

---

## 6. keyforge-protocol (Status: ~95%)
*   [ ] **Task PRO-1: Limited Vec Deserialization (`src/serde_utils.rs`)**
    *   **Gap:** `TransportLimit` error branch.
    *   **Action:** Implement `test_deserialize_vec_limit_hit`. Pass a JSON array with 100,001 items to a struct using `deserialize_limited_vec`.

---

## 7. Execution Loop (Sequential & Efficient)

1.  **Iterate:** `cargo test -p <crate> --lib <path>::tests::<task_test_name>`
2.  **Verify:** `cargo test -p <crate>` (Once all crate tasks are done).
3.  **Audit:** `just cover <crate>` (Final reconciliation).
