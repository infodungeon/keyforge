---
name: arch-003-physics-analysis-determinism
status: open
created: '2026-02-06T15:58:10.208Z'
updated: '2026-02-06T16:12:00.000Z'
progress: 0
totalTasks: 5
completedTasks: 0
---

## Overview
Converted from PRD: arch-003-physics-analysis-determinism

## Technical Approach
We will transition the analysis layer from 'Post-Accumulation Float Scaling' to 'Deterministic Integer Scaling'. 

1. Accumulation: All movement penalties (SFB, Scissor, Redirect, Roll) will be accumulated as Score (fixed-point i64).
2. Scaling: Normalization factors like 100k-keypresses will be implemented using i128 intermediate math: (AccumulatedValue * ScalingFactor) / TotalFrequency. 
3. Types: AnalysisReport heatmap and penalty_map will transition from Vec<f32> to Vec<Score>. 
4. UI Compatibility: The Display trait for Score will handle the float representation in the UI, keeping the engine core pure.

## Implementation Phases
- Phase 1: Model Hardening. Update AnalysisReport to use Score and MetricSet for all internal fields.
- Phase 2: Kernel Transition. Refactor analyze_layout to use i64 fixed-point accumulation.
- Phase 3: Deterministic Scaling. Implement i128 intermediate math for normalization (norm_100k, norm_pct).
- Phase 4: Oracle Parity. Update integration tests to enforce 0% drift between Ghost and Optimized kernels.
- Phase 5: Verification. Execute Narsil ARCH-003 scan and verify performance parity.

## User Stories (ETS-100x)

### [ARCH-003-01] Standardize AnalysisReport with Score and MetricSet
- **Status:** TODO
- **Description:** Refactor `AnalysisReport` struct to exclusively use `Score` and `MetricSet` types.
- **Acceptance Criteria:** Zero float fields in `AnalysisReport` core.

### [ARCH-003-02] Refactor analyze_layout for i64 Fixed-Point Accumulation
- **Status:** TODO
- **Description:** Refactor `analyze_layout` to identify and convert all accumulation points to `i64` fixed-point.
- **Acceptance Criteria:** `analyze_layout` internal state is integer-only.

### [ARCH-003-03] Implement i128 Intermediate Math for Normalization
- **Status:** TODO
- **Description:** Refactor `norm_100k` and `norm_pct` to leverage `i128` for intermediate products/dividends.
- **Acceptance Criteria:** Final scaled results match scoring kernel bit-for-bit.

### [ARCH-003-04] Enforce 0% Drift in Integration Tests
- **Status:** TODO
- **Description:** Update `analysis_verification.rs` to compare Ghost vs Optimized kernels with strict parity.
- **Acceptance Criteria:** `just verify-parity` passes with zero tolerance.

### [ARCH-003-05] Execute Narsil ARCH-003 Scan
- **Status:** TODO
- **Description:** Run Narsil security/architectural scan to verify ARCH-003 compliance.
- **Acceptance Criteria:** Zero architectural findings in `libs/keyforge-physics`.

## Dependencies
- keyforge-model (Score type)
- keyforge-physics (analyze_layout kernel)
- keyforge-testing-macros (kf_test)
- narsil-mcp (ARCH-003 enforcement)

## Success Criteria
Inherited from PRD

## Implementation Invariants (CRITICAL)

### [INV-ARCH-003-01] Deterministic Normalization
To prevent precision loss and intermediate overflow, all normalization (e.g., Score per 100k keys) MUST follow the `i128` promotion rule:

```rust
// Formula: (Accumulated * Scale + (TotalFreq / 2)) / TotalFreq
// Implementation MUST promote to i128 before multiplication.
```

1. **Promotion:** Cast `Score.raw()` and `Scale` to `i128`.
2. **Product:** Compute product in `i128`.
3. **Rounding:** Use `(product + (divisor / 2)) / divisor` for half-up parity.
4. **Safety:** Explicit zero-check for `TotalFrequency`.

---
*Generated from PRD by GeminiAutoPM MCP Server*
*Original PRD: .claude/prds/arch-003-physics-analysis-determinism.md*
