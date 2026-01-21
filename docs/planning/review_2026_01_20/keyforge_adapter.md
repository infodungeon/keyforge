# Review: keyforge-adapter

**Date:** 2026-01-20

## libs/keyforge-adapter/src/conversion/geometry.rs
- [ ] **Task-adap-rev-001**: Line 24: `is_home` override.
    - **Deficiency**: Overwrites input flag based on row index.
    - **Recommendation**: Respect input.
- [ ] **Task-adap-rev-002**: Line 42: Manual field copying.
    - **Deficiency**: Verbose mapping.
    - **Recommendation**: `From` impl.
- [ ] **Task-adap-rev-003**: Line 100: Inefficient lookup.
    - **Deficiency**: Rebuilds map every call.
    - **Recommendation**: Pass map.

## libs/keyforge-adapter/src/conversion/layout.rs
- [ ] **Task-adap-rev-004**: Line 43: Weak argument stripping.
    - **Deficiency**: `find('(')` only.
    - **Recommendation**: Tokenizer.
- [ ] **Task-adap-rev-005**: Line 59: Unconditional padding.
    - **Deficiency**: Masks configuration errors.
    - **Recommendation**: Strict mode length check.

## libs/keyforge-adapter/src/conversion/config.rs
- [ ] **Task-adap-rev-006**: Line 34: Partial roll bonus.
    - **Deficiency**: Ignores other roll types.
    - **Recommendation**: Map all.