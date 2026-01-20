# Coverage Remediation Plan: The "Batched Verification" Strategy

**Status:** Draft
**Date:** 2026-01-20
**Goal:** Achieve 100% Test Coverage on `libs/*` with minimal idle time.

## 1. The Core Problem & Solution
**Problem:** `cargo tarpaulin` takes 3-5 minutes per crate. Using it as a feedback loop (Red -> Green -> Refactor) is prohibitively slow.
**Solution:** Decouple **Execution Verification** (Fast) from **Coverage Verification** (Slow).

### The Proposed Workflow
1.  **Baseline (Once/Day):** Run full coverage generation to create HTML reports.
2.  **The "Blind" Loop (Iterative):**
    *   **Identify:** Visually inspect the HTML report to find "Red Zones".
    *   **Target:** Select a specific module (e.g., `keyforge-model/src/config/weights.rs`).
    *   **Implement:** Write unit tests specifically targeting the red branches.
    *   **Verify (Fast):** Run `cargo test -p <crate> --lib <module>` (< 10s).
    *   *Repeat* for all modules in the batch.
3.  **Reconciliation (Batch):** Only after completing a significant batch (e.g., an entire crate), run `cargo tarpaulin` to confirm 100% and catch edge cases.

---

## 2. Execution Queue

### Phase 1: The Nucleus (High Risk, High Value)
**Target:** `keyforge-physics` (Current: ~95%) & `keyforge-evolution` (Current: ~0%)

#### Task 1.1: Physics Closure (`keyforge-physics`)
*   **Focus:** `src/verify.rs` and `src/engines/intel_comet_lake.rs`.
*   **Action:** Add specific edge-case tests for:
    *   `DeterministicScorer` overflow branches (using `Corpus` with `u64::MAX`).
    *   `IntelScoringEngine` AVX2 fallback paths (force disable AVX2 via config if possible, or mock architecture checks).
    *   `score_detailed` branches for specific geometries (e.g., "scissor" detection thresholds).

#### Task 1.2: Evolution Initialization (`keyforge-evolution`)
*   **Focus:** `supervisor/optimizer.rs` and `supervisor/annealing.rs`.
*   **Action:** Create "Mock Supervisor" tests.
    *   Test `Optimizer` state transitions (Idle -> Running -> Paused).
    *   Test `Annealing` temperature decay functions in isolation (pure math tests).
    *   **Strategy:** Do not spin up full threads. Test the `step()` functions deterministically.

### Phase 2: The Contract (Data Integrity)
**Target:** `keyforge-model` (Current: ~20%)
*Note: This crate is mostly structs/enums. Coverage here means "Verify Parsing and Validation logic".*

#### Task 2.1: Configuration Parsing
*   **Files:** `src/config/weights.rs`, `src/config/search.rs`.
*   **Action:**
    *   Create `tests/fixtures/config_valid.toml` and `config_invalid.toml`.
    *   Write unit tests that deserialize these into `Weights` structs.
    *   Explicitly test validation failures (e.g., negative weights, out-of-bounds parameters).

#### Task 2.2: Geometry & KLE
*   **Files:** `src/geometry/kle.rs`.
*   **Action:**
    *   Test parsing of standard KLE JSONs (TKL, 60%, ErgoDox).
    *   Verify coordinate mapping logic (X/Y rotation handling).

#### Task 2.3: Keycodes & Layouts
*   **Files:** `src/keycodes.rs`, `src/layout.rs`.
*   **Action:**
    *   Test `Display` and `FromStr` implementations for `KeyCode`.
    *   Test `Layout` validation logic (duplicate keys, missing keys).

### Phase 3: The Infrastructure (Mocking Heavy)
**Target:** `keyforge-infra` & `keyforge-adapter`

#### Task 3.1: Asset Providers
*   **Files:** `src/asset/fs_provider.rs`, `src/asset/caching_provider.rs`.
*   **Action:**
    *   Use `tempfile` crate to create ephemeral directories for `FsProvider` tests.
    *   Mock the backing store for `CachingProvider` to verify cache hits/misses without a real DB.

---

## 3. Tooling Setup (Actionable Items)

1.  **Generate Baseline Report:**
    ```bash
    cargo tarpaulin --workspace --out Html --output-dir target/coverage-report
    ```
    *Open `target/coverage-report/tarpaulin-report.html` in browser to guide work.*

2.  **Fast Test Aliases:**
    *   Use `cargo test -p <pkg> --lib` strictly.

3.  **Coverage Command (Per Package):**
    *   `just cover <package>` (already added).

## 4. Next Steps
1.  **Approve Plan:** Confirm this prioritization.
2.  **Execute Phase 1.1:** Close `keyforge-physics` gaps immediately.
3.  **Execute Phase 2.1:** Scaffold `keyforge-model` config tests.
