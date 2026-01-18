# Remediation Plan: Deconstructing the `compute.rs` God Class

**Target:** `libs/keyforge-physics/src/kernel/compute.rs` (1100 LOC)
**Risk Score:** 6020 (Critical)
**Goal:** Dismantle the single file into a modular, maintainable kernel with clear separation of concerns.

## 1. Architectural Blueprint

We will transition from a Monolithic architecture to a Modular Kernel architecture.

### Current State
`src/kernel/compute.rs` handles:
1.  **State Management:** `PosMap`, `PhysicsScratch` (Memory mapping)
2.  **Pure Physics:** `calculate_flow_cost` (The rules of movement)
3.  **Scoring Loop:** `score_layout` (The hot path O(N^3))
4.  **Differential Scoring:** `calculate_swap_delta` (The optimization path)
5.  **Analysis/Reporting:** `analyze_layout` (The debug path with String allocs)

### Target State
```text
libs/keyforge-physics/src/kernel/
└── compute/                  <-- NEW MODULE DIRECTORY
    ├── mod.rs                (Public API Facade - Re-exports everything)
    ├── state.rs              (Data Structures: PosMap, PhysicsScratch)
    ├── flow.rs               (Pure Math: calculate_flow_cost, helpers)
    ├── scoring.rs            (Hot Path: score_layout)
    ├── delta.rs              (Optimization: calculate_swap_delta)
    ├── analysis.rs           (Reporting: analyze_layout - optimized)
    └── tests.rs              (Unit Tests migrated here)
```

## 2. Execution Tasks (Step-by-Step)

### Phase 1: Infrastructure & State (Low Risk)
*Goal: Establish the new module structure without breaking the build.*

- [x] **Task [REFAC-01]: Create Module Structure**
    - Create directory `libs/keyforge-physics/src/kernel/compute/`.
    - Create empty files: `mod.rs`, `state.rs`, `flow.rs`, `scoring.rs`, `delta.rs`, `analysis.rs`.
    - Update `libs/keyforge-physics/src/kernel/mod.rs` to expose the new module.

- [x] **Task [REFAC-02]: Extract State Containers**
    - Move `PosMap` struct and `impl` to `state.rs`.
    - Move `PhysicsScratch` struct and `impl` to `state.rs`.
    - **Verification:** `cargo check -p keyforge-physics`.

### Phase 2: Pure Logic Extraction (Medium Risk)
*Goal: Isolate the "Rules of Physics" from the execution loops.*

- [x] **Task [REFAC-03]: Extract Flow Math**
    - Move `calculate_flow_cost` to `flow.rs`.
    - Move `get_p_effective` and `get_flow_delta` helpers to `flow.rs`.
    - **Logic Check:** Ensure these functions remain `#[inline(always)]` for performance.

### Phase 3: The "Cold" Path (Analysis) (Medium Risk)
*Goal: Remove the heaviest function (LOC-wise) to declutter the core.*

- [x] **Task [REFAC-04]: Extract Analysis Logic**
    - Move `analyze_layout` to `analysis.rs`.
    - Move `u16_to_char` helper to `analysis.rs`.
    - **Optimization:** Refactor `u16_to_char` to write to a `fmt::Write` buffer or return `char` instead of allocating `String`, fixing the performance finding from the audit.

### Phase 4: The "Optimization" Path (Delta) (High Risk)
*Goal: Isolate the complex logic used by the Genetic Algorithm.*

- [x] **Task [REFAC-05]: Extract Delta Logic**
    - Move `calculate_swap_delta` to `delta.rs`.
    - Ensure it imports `flow::get_flow_delta`.

### Phase 5: The "Hot" Path (Scoring) (High Risk)
*Goal: Isolate the critical scoring loop.*

- [x] **Task [REFAC-06]: Extract Scoring Loop**
    - Move `score_layout` to `scoring.rs`.
    - This file should now be very small (<100 LOC), focusing purely on iterating through the corpus and summing costs.

### Phase 6: Test Migration & Verification
*Goal: Ensure no regressions.*

- [x] **Task [REFAC-07]: Migrate Tests**
    - Move the `mod tests` module from `compute.rs` to `compute/tests.rs` (or split them into their respective files if preferred).
    - **Verification:** Run `cargo test -p keyforge-physics`. (Passed after fixing `sorted_unique_keys` logic in `compiler.rs`)
    - **Benchmark:** Run `cargo bench -p keyforge-physics` (once benchmarks are restored) to ensure no perf regression from module splitting.

## 3. Benefits of this Refactor
1.  **Cognitive Load:** Developers can reason about "Math" (`flow.rs`) separately from "Memory" (`state.rs`).
2.  **Compile Times:** Changing the reporting logic (`analysis.rs`) won't force a recompile of the optimization logic (`delta.rs`).
3.  **Safety:** `state.rs` can be heavily audited for `unsafe` (if any), while `flow.rs` can be purely safe math.
4.  **Profiling:** It becomes much easier to profile `scoring.rs` in isolation.
