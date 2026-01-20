# Libs 100% Unit Test Coverage: Exhaustive Execution Plan

**Objective:** Zero "Logic Dark Matter" in the `libs/` directory.
**Methodology:** 
1.  **Branch Exhaustion:** Identify every `if/else`, `match` arm, and `checked_*` arithmetic operation.
2.  **Error Injection:** Force every `Result::Err` and `Option::None` variant.
3.  **Efficiency:** Micro-targeted testing via `cargo test -p <crate> --lib <path>` to keep the loop under 15 seconds.

---

## 1. keyforge-physics (The Math Kernel)
*Status: ~95% | Target: Finish remaining arithmetic and hardware branches.*

### 1.1 Core Compute (`src/kernel/compute/`)
*   [ ] **PHY-1.1.1: Trigram Overlap Branches (`delta.rs`)**
    *   Target: Branches where trigram overlaps are calculated during a swap.
    *   Test: `test_delta_trigram_overlap_variants`. Construct a layout with 3 specific keys and perform a swap that affects only the 1st or 3rd position of a trigram.
*   [ ] **PHY-1.1.2: Flow Cost Edge Cases (`flow.rs`)**
    *   Target: `calculate_flow_cost` branches for `dir1 == 0` or `dir2 == 0`.
    *   Test: `test_flow_cost_zero_dir`. Use keys on the same finger but different rows where the finger difference is 0.
*   [ ] **PHY-1.1.3: Scoring Saturation (`scoring.rs`)**
    *   Target: `checked_add` fallbacks in monogram/bigram loops.
    *   Test: `test_scoring_accumulator_saturation`. Use `EngineContext` with `i64::MAX / 2` costs.

### 1.2 Engines (`src/engines/`)
*   [ ] **PHY-1.2.1: Intel Prefetch Config (`intel_comet_lake.rs`)**
    *   Target: Logic that toggles `use_prefetch` in the `IntelEngineConfig`.
    *   Test: `test_intel_prefetch_toggle`. Run a benchmark-style loop with prefetch ON and OFF to ensure both branches execute.
*   [ ] **PHY-1.2.2: Generic Engine Trait Parity (`generic.rs`)**
    *   Target: Any remaining `Result` mappings in the `ScoringEngine` trait implementation.

### 1.3 Verification (`src/verify.rs`)
*   [ ] **PHY-1.3.1: Trigram Accumulation Overflow**
    *   Target: `checked_mul` and `checked_add` in the trigram loop.
    *   Test: `test_oracle_trigram_overflow`. Inject `u32::MAX` frequency and a massive `penalty_redirect` into the `DeterministicScorer`.

---

## 2. keyforge-model (The Semantic Contract)
*Status: ~85% | Target: 100% Validation and Serde coverage.*

### 2.1 Geometry (`src/geometry/`)
*   [ ] **MOD-2.1.1: KLE Malformed JSON (`kle.rs`)**
    *   Target: `serde_json::from_str` errors and `keyboard.keys` empty check.
    *   Test: `test_kle_parse_errors`. Pass `[]`, `{}`, and `[[{"r": "invalid"}]]` to `parse_kle_json`.
*   [ ] **MOD-2.1.2: Coordinate Clamping (`mod.rs`)**
    *   Target: `KeyboardGeometry::validate` bounds checking.
    *   Test: `test_geometry_bounds_clamping`. Create a geometry with keys at `x: 1000`.

### 2.2 Config (`src/config/`)
*   [ ] **MOD-2.2.1: Search Parameter Exhaustion (`search.rs`)**
    *   Target: Every `if` check in `SearchParams::validate`.
    *   Test: `test_search_params_exhaustive_validation`. Loop through every parameter (temp, steps, limits) and set it to 0, negative, or `MAX+1`.
*   [ ] **MOD-2.2.2: Weight Category Logic (`weights.rs`)**
    *   Target: `allowed_hand_balance_deviation` and `get_comfortable_scissors`.
    *   Test: `test_weight_calc_logic`. Verify the math for balance deviation correctly clamps at 0.0.

---

## 3. keyforge-infra (The Operational Shell)
*Status: ~70% | Target: Full coverage for IO providers and net sync.*

### 3.1 Asset Providers (`src/asset/`)
*   [ ] **INF-3.1.1: Valkey Provider (0% Coverage)**
    *   Target: All methods in `ValkeyProvider`.
    *   Test: `test_valkey_provider_full`. Implement `MockCoordinator` and verify `load`, `list_keyboards`, and `get_corpus_hash` logic.
*   [ ] **INF-3.1.2: Caching Eviction (`caching_provider.rs`)**
    *   Target: `handle_fs_event` for `corpora` and `weights` categories.
    *   Test: `test_cache_invalidation_by_category`. Trigger events for `system/corpora/` and `system/weights/` paths and verify granular cache clears.

### 3.2 Sync & Networking (`src/net/`)
*   [ ] **INF-3.2.1: Sync Stats & Errors (`sync.rs`)**
    *   Target: `SyncStats` collection during `run_sync` failures.
    *   Test: `test_sync_error_accumulation`. Mock 404s for 50% of files and verify `stats.errors` count.

---

## 4. keyforge-evolution (The Search Engine)
*Status: ~90% | Target: Temperature underflow and state transitions.*

### 4.1 Annealing (`src/supervisor/`)
*   [ ] **EVO-4.1.1: Temperature Underflow (`annealing.rs`)**
    *   Target: `if state.temperature < TEMP_UNDERFLOW_THRESHOLD`.
    *   Test: `test_annealing_underflow`. Set `end_temp` to `1e-40` and verify temperature hits exactly `0.0`.
*   [ ] **EVO-4.1.2: Reheating Math (`state.rs`)**
    *   Target: `reheat_from_best` logic.
    *   Test: `test_state_reheat_math`. Verify temperature is correctly spiked by the `reheat_factor`.

---

## 5. Remaining Libraries (Full Sweep)

### 5.1 keyforge-persistence (0% on Compiler)
*   [ ] **PER-5.1.1: Query Recording Failure.**
    *   Test: `test_persistence_compiler_io_error`. Verify that if writing to the WAL fails, the compiler returns a clean error.

### 5.2 keyforge-protocol (Serde Utils)
*   [ ] **PRO-5.2.1: Vec Size Clamping.**
    *   Test: `test_deserialize_clamping`. Pass an array of 100,001 items and verify the `TransportLimit` error.

### 5.3 keyforge-export (0% Coverage)
*   [ ] **EXP-5.3.1: QMK/ZMK/VIA Macros.**
    *   Test: `test_export_formats_exhaustive`. Run one layout through all three exporters and verify string contents match the expected firmware syntax.

---

## 6. Execution Workflow (Strict Efficiency)

For each task:
1.  **Read:** `read_file` the target module.
2.  **Write:** `replace` or `write_file` to add the specific test case.
3.  **Micro-Verify:** `cargo test -p <crate> --lib <module>::tests::<test_name>`
4.  **Batch-Verify:** Once a crate section is done, run `cargo test -p <crate>`.
5.  **Reconcile:** `just cover <crate>` to confirm 100%.
