# Review: keyforge-compute

**Date:** 2026-01-20

## libs/keyforge-compute/src/hardware.rs
- [ ] **Task-comp-rev-001**: Line 54: Manual cache calculation.
    - **Deficiency**: Fragile manual math.
    - **Recommendation**: Use library helpers.
- [ ] **Task-comp-rev-002**: Line 31: Default values.
    - **Deficiency**: Silent defaults.
    - **Recommendation**: Log warning.

## libs/keyforge-compute/src/biometrics.rs
- [ ] **Task-comp-rev-003**: Line 36: Hardcoded 150ms.
    - **Deficiency**: Typing speed bias.
    - **Recommendation**: Relative normalization.
- [ ] **Task-comp-rev-004**: Line 33: Threshold 5.
    - **Deficiency**: Hardcoded significance.
    - **Recommendation**: Configurable.