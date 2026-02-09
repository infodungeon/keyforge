# ADR 026: Standardize Fixed-Point Arithmetic in Physics Kernel

## Status
Accepted

## Context
The `libs/keyforge-physics` kernel and `libs/keyforge-model` currently use a mix of `FixedWeight` (i32, 1000 scale) and `Score` (i64, 1,000,000 scale) for representing costs and weights. Furthermore, the `Rubric` configuration relies on `f32` for input and serialization, converting to `FixedWeight` via `from_f32`. This introduces potential non-determinism in the configuration loading phase and kernel calculations (where intermediate `f64` scaling is sometimes used or simulated), violating **ARCH-003 (Deterministic Physics)**.

To ensure bit-perfect determinism across all platforms and compilation targets (including WASM), the physics engine must eliminate all reliance on floating-point arithmetic in its logic paths.

## Decision
1.  **Unify on `Score`**: Deprecate the usage of `FixedWeight` within the `Rubric` struct. All configuration weights in `Rubric` and `RawRubric` will be migrated to `Score` (i64, 1,000,000 scale).
2.  **Eliminate Kernel Floats**: The `keyforge-physics` kernel (specifically `mechanics.rs`, `geometry.rs`, and `costs.rs`) must be refactored to use `Score` exclusively. Intermediate calculations involving products of `Score` values will use `i128` to prevent overflow before scaling back to the `Score` domain.
3.  **Strict Builders**: `RubricBuilder` will be updated to primarily support `Score` inputs. Floating-point convenience methods will be marked as such and must implement a strictly deterministic conversion (e.g., parsing/rounding guarantees) or be restricted to non-production/test scopes if they cannot be guaranteed.
4.  **Serialization**: `Rubric` serialization will default to the raw integer representation or a deterministic decimal string format to avoid IEEE 754 parsing ambiguities.

## Consequences

### Positive
*   **Bit-Perfect Determinism**: Guarantees identical scoring results across all architectures (x86, ARM, WASM).
*   **Higher Precision**: Increases configuration weight precision from 0.001 to 0.000001.
*   **Type Simplification**: Reduces cognitive load by using `Score` consistently across the domain model.
*   **Compliance**: Satisfies ARCH-003.

### Negative
*   **Breaking Change**: Existing serialized `Rubric` JSONs (if any exist in the wild using `f32` values) may need migration or a compatibility layer.
*   **Memory Usage**: `Rubric` size increases slightly (i32 -> i64 fields), but this is negligible for a singleton configuration struct.
*   **Verbosity**: Integer-based configuration in tests might be more verbose (e.g., `Score::from_raw(500_000)` vs `0.5`), requiring helper traits.
