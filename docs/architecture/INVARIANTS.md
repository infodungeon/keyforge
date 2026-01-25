# KeyForge Architectural Invariants

This document defines the technical "Law" of the KeyForge system. Deviations from these rules are considered architectural failures.

## 1. Performance Invariants (Allocation Scrub)
- **PERF-001: Shared Ownership for Large Data**: Any array or slice exceeding 1,024 elements (e.g., Corpus frequencies, Cost Matrices) MUST be wrapped in `Arc<[T]>`.
- **PERF-002: Zero-Clone Hot Loops**: No `.clone()` calls are permitted within the `libs/keyforge-physics` scoring kernels or `libs/keyforge-evolution` optimization loops.
- **PERF-003: Immutable Core Assets**: Once compiled, the `EngineContext` and its constituent data structures MUST be immutable.

## 2. Type-Safety & Error Invariants
- **TYPE-001: Panic-Free Production**: The use of `unwrap()` or `expect()` is strictly forbidden in production code (`apps/` and `libs/`).
- **TYPE-002: Total Error Propagation**: All fallible operations must return a `Result` and use the `ForgeError` or crate-specific error type.
- **TYPE-003: Correct-by-Construction**: Use Newtypes (e.g., `KeyIndex`, `Score`) to prevent primitive obsession and argument-swapping.

## 3. Testing Invariants
- **TEST-001: Explicit Error Handling**: Tests MUST NOT use `unwrap()`. Use `?` and return `Result<(), Box<dyn Error>>` (facilitated by `#[kf_test]`).
- **TEST-002: Observable Mocking**: Mock filesystems MUST be explicitly mapped in `ASSET_RESOLUTION.md`. Guessing paths is a systemic failure.
- **TEST-003: Parity Oracles**: Optimized engines MUST be verified against the `GhostScorer` (Oracle) for bit-perfect parity in fixed-point math.
