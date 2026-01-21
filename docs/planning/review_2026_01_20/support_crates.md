# Review: Support Crates

**Date:** 2026-01-20

## libs/keyforge-security/src/lib.rs
- [ ] **Task-sec-rev-001**: Line 135: Ambiguous errors.
    - **Deficiency**: Generic Encoding Error.
    - **Recommendation**: Specific context.

## libs/keyforge-testing/src/lib.rs
- [ ] **Task-test-rev-001**: Line 42: Duplicate paths.
    - **Deficiency**: Redundant logic from infra.
    - **Recommendation**: Share code.

## libs/keyforge-wasm/src/loader.rs
- [ ] **Task-wasm-rev-001**: Line 135: Single source only.
    - **Deficiency**: Ignores multiple corpora.
    - **Recommendation**: Merge logic.