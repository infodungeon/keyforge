# Planning: Multi-Tiered Scoring Engine Refactor (Engineering Truth)

## Objective
Refactor `keyforge-physics` to provide a non-lossy, bit-perfect "Exact" implementation and logically consistent "Optimized" implementations. 

**Core Mandate:** Engineering Truth. An overflow or NaN is a corruption of state (Bad Code or Bad Data). The system must fail explicitly rather than clamping or returning corrupt values.

## Technical Strategy

### 1. The `ScoringEngine` Trait
The trait explicitly returns a `Result` to signal logical failures.
```rust
pub trait ScoringEngine {
    fn score(&self, layout: &Layout) -> Result<Score, PhysicsError>;
    fn name(&self) -> &'static str;
}
```

### 2. The Normalization Standard (Precision Rule)
To achieve bit-perfect parity, we define the **Point of Finalization**:
*   Intermediate geometric math (Distance, Effort, Flow) MUST be performed in `f64`.
*   Finalization to `Score` (fixed-point `i64`) MUST happen at the **Kernel Boundary** (e.g., after calculating a full Bigram cost but before adding it to the total layout accumulator).
*   Summation of these finalized `Score` values MUST use `checked_add`.

## Detailed Task List

### Phase 1: Foundation of Truth (Types & Errors)
*   [x] **Refactor `Score` (`libs/keyforge-model/src/types.rs`)**
    *   [x] Implement `checked_add` and `checked_mul`.
    *   [x] Update `from_f32` to return `Result<Score, String>`.
*   [x] **Define `PhysicsError` (`libs/keyforge-physics/src/error.rs`)**
    *   [x] Add `ScoreOverflow { context: String }` and `InvalidInput` variants.
*   [x] **Update Trait (`libs/keyforge-physics/src/lib.rs`)**
    *   [x] Change `score` signature to return `Result<Score, PhysicsError>`.
*   [x] **Blast Radius Management**: Update all call sites in `keyforge-evolution`, `keyforge-compute`, and `keyforge-cli` to handle the new `Result` return.

### Phase 2: Logic Synchronization (Kernel Audit)
*   [x] **The Normalization Implementation**: Standardize the `f64` -> `i64` conversion point across all engines.
*   [x] **Matrix Integrity**: Audit the `CostMatrix` builder to ensure physical distances are calculated using `f64` and same-hand rules.
*   [x] **Sync Travel Logic**: Update all engines to only calculate distance for same-hand bigrams.
*   [x] **Sync SFB Logic**: Audit `Weak Lateral`, `Diagonal`, and `Step` branches for exact parity.

### Phase 3: Oracle & Exact Engine Alignment
*   [x] **Refactor Oracle (`libs/keyforge-physics/src/verify.rs`)**
    *   [x] Update `DeterministicScorer` to return `Result`.
*   [x] **Refactor Exact Engine (`libs/keyforge-physics/src/engines/exact.rs`)**
    *   [x] Achieve bit-perfect (0 drift) match with `DeterministicScorer`.

### Phase 4: Optimized Engine Hardening
*   [x] **Refactor Generic/Intel Engines**:
    *   [x] Ensure logical parity with the audited rules.
    *   [x] Implement checked arithmetic and `Result` propagation in all scoring kernels.
    *   [ ] **SIMD Safety Strategy**: Implement post-block overflow detection in SIMD kernels to maintain performance while guaranteeing "No Clamping." (Currently using scalar fallbacks with error handling).
    *   [ ] **Performance Audit**: Benchmark `checked_add` overhead in SIMD loops.

### Phase 5: Domain Invariant Enforcement
*   [x] **Proptest Constraints (`libs/keyforge-model/src/testing.rs`)**: Constrain coordinates/weights to physical ranges.
*   [x] **Safe Math Cleanup**: Replace `abs()` with `unsigned_abs()` in all coordinate and index math to prevent negation overflows.

### Phase 6: Observability & Documentation
*   [x] **Doc Sync**: Update `docs/architecture/11_SCORING_LOGIC.md` and `00_MANIFESTO.md`.
*   [x] **Error Contextualization**: Ensure overflows report which char/bigram caused the failure.

### Phase 7: Verification & CI
*   [x] **Parity Verification**: `Exact` engine = 0 drift. Verified via proptests.
*   [x] **Overflow Verification**: Test case for `Err(ScoreOverflow)` verified in `kernel::compute::tests`.
*   [x] **CI Guardrail**: Add a build step that runs parity tests and fails on any `Exact` drift.
*   [x] **Best-Candidate Re-validation**: Integrate `ExactScoringEngine` re-validation in the evolution loop.

## Verification Criteria
*   **Exact:** `assert_eq!(engine.score(layout)?, oracle.score(layout)?)` -> **ACHIEVED BIT-PERFECT PARITY**
*   **Integrity:** Any layout that causes a math error is rejected, not clamped. -> **ENFORCED VIA CHECKED ARITHMETIC**