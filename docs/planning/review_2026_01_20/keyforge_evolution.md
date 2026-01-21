# Review: keyforge-evolution

**Date:** 2026-01-20
**Tier:** 1

## libs/keyforge-evolution/src/lib.rs
- [ ] **Task-evol-rev-001**: Line 30: `bool` abort signal.
    - **Deficiency**: Silent abort reason.
    - **Recommendation**: Return `Result/Enum`.

## libs/keyforge-evolution/src/errors.rs
- [ ] **Task-evol-rev-002**: Line 30: Stringly typed `Config` error.
    - **Deficiency**: Hard to handle programmatically.
    - **Recommendation**: Structured variants.

## libs/keyforge-evolution/src/supervisor/annealing.rs
- [ ] **Task-evol-rev-003**: Line 52: `ips` unit confusion.
    - **Deficiency**: Calculates MOPS but labels IPS.
    - **Recommendation**: Rename/Scale.
- [ ] **Task-evol-rev-004**: Line 57: Layout cloning.
    - **Deficiency**: Deep copy in reporting loop.
    - **Recommendation**: COW/Arc.
- [ ] **Task-evol-rev-005**: Line 185: Concurrency coupling.
    - **Deficiency**: Scope manages both worker/reporter; reporter panic undefined.
    - **Recommendation**: Explicit handles.
- [ ] **Task-evol-rev-006**: Line 210: Abort check frequency.
    - **Deficiency**: Coupled to report interval.
    - **Recommendation**: Decoupled check (every N steps).

## libs/keyforge-evolution/src/supervisor/optimizer.rs
- [ ] **Task-evol-rev-007**: Line 104: Brittle pin resolution.
    - **Deficiency**: `position` finds first match; assumes uniqueness.
    - **Recommendation**: Validate uniqueness.
- [ ] **Task-evol-rev-008**: Line 145: Redundant re-scoring.
    - **Deficiency**: Re-scores even if engine is Exact.
    - **Recommendation**: Check capabilities.

## libs/keyforge-evolution/src/supervisor/state.rs
- [ ] **Task-evol-rev-009**: Line 43: `pos_map` size scaling.
    - **Deficiency**: Sized by max keycode value, not count.
    - **Recommendation**: Sparse map or fixed size.
- [ ] **Task-evol-rev-010**: Line 133: Inefficient reheating.
    - **Deficiency**: Rebuilds pos_map.
    - **Recommendation**: Cache best_pos_map.
- [x] **Task-evol-rev-011**: Line 82: `pos_map` bounds assumption.
    - **Deficiency**: Ignores codes outside initial range.
    - **Recommendation**: Validate range.

## libs/keyforge-evolution/src/supervisor/traits.rs
- [ ] **Task-evol-rev-012**: Line 43: Closed `MutationAction`.
    - **Deficiency**: Hardcoded enum limits extensibility.
    - **Recommendation**: Intent pattern.

## libs/keyforge-evolution/src/supervisor/strategies/group.rs
- [ ] **Task-evol-rev-013**: Line 118: Cloning in scratch.
    - **Deficiency**: `clone()` inside scratch closure allocates.
    - **Recommendation**: Slice API.
- [ ] **Task-evol-rev-014**: Line 106: `copy_from_slice` overhead.
    - **Deficiency**: Full copy for partial change.
    - **Recommendation**: Patch/Rollback.
- [ ] **Task-evol-rev-015**: Line 41: Magic `p_swap` constants.
    - **Deficiency**: Hardcoded adaptive logic.
    - **Recommendation**: Config parameters.