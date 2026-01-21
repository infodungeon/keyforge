# Review: Dependencies

**Date:** 2026-01-20

## Cargo.toml (Workspace)
- [ ] **Task-deps-rev-001**: Missing workspace dependencies.
    - **Deficiency**: `serde`, `tokio` defined in individual crates.
    - **Recommendation**: Unify in root.

## libs/keyforge-protocol/Cargo.toml
- [ ] **Task-deps-rev-002**: `anyhow` in lib.
    - **Deficiency**: Lazy error handling in contract.
    - **Recommendation**: Remove.

## libs/keyforge-physics/Cargo.toml
- [ ] **Task-deps-rev-003**: Inconsistent versions.
    - **Deficiency**: `rand 0.9` vs others.
    - **Recommendation**: Align.