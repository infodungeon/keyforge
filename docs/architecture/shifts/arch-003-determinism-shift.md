# Architectural Shift: Bit-Perfect Determinism (ARCH-003)

**Status:** Implementation Phase 2 (Mapping)
**Track:** #150

## C4 Container Diagram: Scoring Data Flow

```mermaid
C4Container
    title Bit-Perfect Scoring Flow (Post-Shift)

    Container_Boundary(model, "keyforge-model (Tier 1)") {
        Component(score, "Score (i64)", "Fixed-point units")
        Component(rubric, "Rubric", "Strongly-typed weights")
    }

    Container_Boundary(physics, "keyforge-physics (Tier 2)") {
        Component(kernel, "Mechanics Kernel", "Pure integer math (i128 scaling)")
        Component(analysis, "Analysis Layer", "Deterministic Normalization")
    }

    Container_Boundary(wasm, "keyforge-wasm (Tier 3)") {
        Component(bindings, "WASM Bindings", "Exposes AnalysisReport")
    }

    Rel(rubric, kernel, "Passes weights as Score")
    Rel(kernel, analysis, "Accumulates Score")
    Rel(analysis, wasm, "Returns AnalysisReport (Score-based)")
```

## Impact Analysis Summary

1. **libs/keyforge-model**:
    - `RawRubric` fields migration from `f32` to `Score`.
    - `Weight` type alignment with `Score`.
2. **libs/keyforge-physics**:
    - `calculate_pair_cost` refactor to use `i128` intermediate products.
    - `integer_sqrt_i128` remains the primary rounding-safe root.
    - `deterministic_normalize` replaces any remaining `f64` normalization.
3. **libs/keyforge-wasm**:
    - `analyze_layout` updated to handle `Score` types in serialization.
    - Verification of `1.0` float weights in `load_corpus` calls.

## Verification Strategy

- **Oracle Parity**: Use `DeterministicScorer` to verify optimized engine outputs against the pure kernel.
- **Bit-Identical Snapshots**: Compare `AnalysisReport` JSON output between local and CI environments.
