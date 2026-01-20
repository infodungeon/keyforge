# Comprehensive Libs Coverage Roadmap (100% Target)

**Objective:** Achieve 100% line and branch coverage for every crate in the `libs/` and `corpora/` directories.
**Doctrine:** Explicit testing of every `Result` variant, every `match` arm, and every hardware-dependent fallback path.

---

## 1. keyforge-physics (The Core Kernel)
*Current Status: ~95% | Gap: Error paths, SIMD fallbacks, Edge-case metrics.*

### src/verify.rs
*   [ ] **Task PHY-1.1: Monogram Accumulation Overflow**
    *   Test: `test_mono_overflow`. Use `Corpus` with code frequency near `u64::MAX`. Assert `PhysicsError::ScoreOverflow`.
*   [ ] **Task PHY-1.2: Bigram Modifier Overflow**
    *   Test: `test_bigram_mod_overflow`. Add a sequence modifier to `CostModel` that pushes `Score` past `i64::MAX`.
*   [ ] **Task PHY-1.3: Trigram Frequency Scaling Overflow**
    *   Test: `test_trigram_scale_overflow`. Similar to monogram, but for trigram frequency multiplication.
*   [ ] **Task PHY-1.4: Geometric Math Boundaries**
    *   Test: `test_invalid_geometric_input`. Inject `NaN` or `Infinity` into `kb.spatial_cache` (via mock or manual construction) and verify `PhysicsError::InvalidInput` in `calculate_pair_cost`.
*   [ ] **Task PHY-1.5: Static Cost Resolution Branch Exhaustion**
    *   Test: `test_resolve_static_branches`. Cover all `match` arms for `FingerIndex` (Thumb vs Standard) and `Zone` selection (Inner/Outer/Base).

### src/engines/intel_comet_lake.rs
*   [ ] **Task PHY-2.1: Scalar Fallback Logic Parity**
    *   Test: `test_intel_scalar_parity`. Directly invoke `score_layout_scalar` and compare results with the default engine entry point to ensure the non-AVX2 path is logically identical.
*   [ ] **Task PHY-2.2: Config Default Verification**
    *   Test: `test_intel_config_defaults`. Verify `IntelEngineConfig::default()` values.

### src/analysis/heuristics.rs
*   [ ] **Task PHY-3.1: Zero Score Boundary**
    *   Test: `test_heuristics_zero_score`. Verify early return when layout has no valid keys to swap.
*   [ ] **Task PHY-3.2: Multimap Key Collisions**
    *   Test: `test_suggest_swaps_multi_mapped`. Verify heuristics handle keys that map to multiple positions (if applicable).

---

## 2. keyforge-model (The Data Contract)
*Current Status: ~20% | Gap: Parsing logic, Serialization, Constants.*

### src/geometry/kle.rs
*   [ ] **Task MOD-1.1: Standard Keyboard Parsing**
    *   Test: `test_parse_ansi_104`. Verify layout mapping for a full-size board.
*   [ ] **Task MOD-1.2: Rotation & Transform Logic**
    *   Test: `test_kle_rotation_parsing`. Verify `r`, `rx`, `ry` fields correctly translate to cartesian coordinates.
*   [ ] **Task MOD-1.3: Error Handling**
    *   Test: `test_invalid_kle_format`. Verify error on missing required fields in KLE JSON.

### src/config/weights.rs
*   [ ] **Task MOD-2.1: Round-Trip Serde**
    *   Test: `test_rubric_toml_parity`. Serialize `Rubric` to TOML, modify, deserialize, and assert equality.
*   [ ] **Task MOD-2.2: Weight Validation**
    *   Test: `test_weight_validation_bounds`. Ensure negative weights or zero denominators in specific formulas trigger validation errors.

### src/keycodes.rs
*   [ ] **Task MOD-3.1: Mapping Completeness**
    *   Test: `test_keycode_conversions`. Verify `FromStr` and `Display` for every variant in the `KeyCode` enum.
*   [ ] **Task MOD-3.2: QMK Parity**
    *   Test: `test_qmk_keycode_export`. Verify export logic for QMK-compatible C headers.

---

## 3. keyforge-evolution (The Search Engine)
*Current Status: ~0% | Gap: All core logic.*

### src/supervisor/optimizer.rs
*   [ ] **Task EVO-1.1: Life Cycle Transitions**
    *   Test: `test_optimizer_lifecycle`. Transition through `Idle -> Running -> Paused -> Terminated`. Assert invariants at each step.
*   [ ] **Task EVO-1.2: Snapshot/Restore**
    *   Test: `test_optimizer_checkpointing`. Save optimizer state to a buffer and reload it; verify search resumes from the same generation.

### src/supervisor/annealing.rs
*   [ ] **Task EVO-2.1: Probability Curving**
    *   Test: `test_annealing_probabilities`. Verify that as temperature decreases, the probability of accepting worse moves strictly decreases.
*   [ ] **Task EVO-2.2: Zero-Temp Boundary**
    *   Test: `test_quench_logic`. Verify behavior when temperature reaches exactly 0.0 (only improvements allowed).

---

## 4. keyforge-infra (The Operational Shell)
*Current Status: ~0% | Gap: FS/IO, Cache eviction, Config.*

### src/asset/fs_provider.rs
*   [ ] **Task INF-1.1: Directory Traversal Safety**
    *   Test: `test_fs_provider_sandbox`. Attempt to read files outside the asset root; assert access denied.
*   [ ] **Task INF-1.2: Concurrent Reads**
    *   Test: `test_fs_provider_concurrency`. Stress test multiple threads reading the same asset file via the provider.

### src/asset/caching_provider.rs
*   [ ] **Task INF-2.1: TTL Eviction**
    *   Test: `test_cache_expiration`. Mock a time source and verify assets are evicted from cache after TTL.
*   [ ] **Task INF-2.2: Memory Pressure**
    *   Test: `test_cache_lru_eviction`. Load assets until the memory limit is hit; verify Least Recently Used are dropped.

---

## 5. keyforge-compute (The Pipeline)
*Current Status: ~0% | Gap: Pipeline construction, Registry.*

### src/builder.rs
*   [ ] **Task CMP-1.1: Invalid Stage Sequences**
    *   Test: `test_builder_validation`. Attempt to build a pipeline where a stage requires inputs that the previous stage does not provide.
*   [ ] **Task CMP-1.2: Parallel Execution Scaling**
    *   Test: `test_pipeline_threading`. Verify `ComputePipeline` correctly utilizes the configured number of threads.

---

## 6. Remaining Libraries (The Tail)

### keyforge-persistence
*   [ ] **Task PER-1.1: SQLx Offline Parity**. Verify that queries in `.sqlx` match the repository trait methods.
*   [ ] **Task PER-1.2: Transaction Rollbacks**. Test that failed repository operations correctly roll back database state.

### keyforge-protocol
*   [ ] **Task PRO-1.1: API Versioning**. Test that old JSON payloads can still be deserialized into current structs (backward compatibility).
*   [ ] **Task PRO-1.2: Error Code Mapping**. Verify that internal `PhysicsError` maps correctly to `ForgeError` HTTP codes.

### keyforge-security
*   [ ] **Task SEC-1.1: JWT Verification**. Test expired, malformed, and valid tokens.
*   [ ] **Task SEC-1.2: Role-Based Access**. Verify permissions for `Admin` vs `User` roles on specific API endpoints.

### keyforge-wasm
*   [ ] **Task WAS-1.1: Memory Buffer Exchange**. Test that large layout arrays can be passed between JS and Rust without corruption.

### keyforge-adapter
*   [ ] **Task ADP-1.1: HTTP Retry Logic**. Mock a 503 error and verify the client retries with exponential backoff.

### keyforge-runner
*   [ ] **Task RUN-1.1: Agent WAL Recovery**. Crash the runner mid-job and verify it resumes from the Write-Ahead Log.

### keyforge-core
*   [ ] **Task COR-1.1: Newtype Invariants**. Test that `KeyIndex` and `FingerIndex` cannot be constructed with out-of-bounds values.

### corpora/openbookcorpus
*   [ ] **Task CRP-1.1: Corpus Cleaning**. Test regex filters against dirty text (HTML tags, non-standard punctuation).

---

## 7. Final Verification Suite
*   [ ] **Task ALL-1.1: Cross-Crate Integration Test**. A single "smoke test" that runs a full evolution job from CLI to Engine and back.
*   [ ] **Task ALL-1.2: Tarpaulin Validation**. Execute `cargo tarpaulin --workspace` for the final 100% certificate.
