# Review: keyforge-contract (Protocol & Model)

**Date:** 2026-01-20
**Tier:** 2

## libs/keyforge-protocol/src/lib.rs
- [ ] **Task-prot-rev-001**: Versioning inconsistencies.
    - **Deficiency**: Min versions hardcoded to 1, Protocol is 2.
    - **Recommendation**: Document policy.

## libs/keyforge-protocol/src/assets.rs
- [ ] **Task-prot-rev-002**: Line 43: `BiometricSample` uses `f64`.
    - **Deficiency**: Mismatch with `f32` engine weights.
    - **Recommendation**: Standardize precision.

## libs/keyforge-protocol/src/node.rs
- [ ] **Task-prot-rev-003**: Line 32: `ips` vs MOPS.
    - **Deficiency**: Telemetry likely reports MOPS as IPS.
    - **Recommendation**: Rename.
- [ ] **Task-prot-rev-004**: Line 52: Signed `cores`.
    - **Deficiency**: `i32` for core count.
    - **Recommendation**: `u32`.
- [ ] **Task-prot-rev-005**: Line 54: Signed cache size.
    - **Deficiency**: `Option<i32>`.
    - **Recommendation**: `u32`.

## libs/keyforge-protocol/src/job.rs
- [ ] **Task-prot-rev-006**: Line 114: Unvalidated IDs.
    - **Deficiency**: `parents` string format not checked.
    - **Recommendation**: Validation regex/length.

## libs/keyforge-model/src/config/weights.rs
- [ ] **Task-prot-rev-007**: Line 131: Arbitrary 50k limit.
    - **Deficiency**: Hardcoded safety cap.
    - **Recommendation**: Workspace constant.
- [ ] **Task-prot-rev-008**: Line 254: Hardcoded 0.5 balance.
    - **Deficiency**: Assumes symmetry.
    - **Recommendation**: Keyboard definition target.
- [ ] **Task-prot-rev-009**: Line 260: Brittle scissor parse.
    - **Deficiency**: Assumes single-digit fingers (ASCII math).
    - **Recommendation**: Robust parsing.

## libs/keyforge-model/src/config/search.rs
- [ ] **Task-prot-rev-010**: Line 150: Hardcoded 0.0001 threshold.
    - **Deficiency**: Magic number for underflow risk.
    - **Recommendation**: Named constant.

## libs/keyforge-model/src/geometry/mod.rs
- [ ] **Task-prot-rev-011**: Line 144: Hardcoded home row 1.
    - **Deficiency**: Fails for non-standard boards.
    - **Recommendation**: Explicit definition.
- [ ] **Task-prot-rev-012**: Line 173: Slot parity check.
    - **Deficiency**: Forces every key into a slot.
    - **Recommendation**: Allow "ignored" keys.

## libs/keyforge-model/src/geometry/kle.rs
- [ ] **Task-prot-rev-013**: Line 43: Fragile split heuristic.
    - **Deficiency**: Guesses hand based on gap.
    - **Recommendation**: Explicit config.
- [ ] **Task-prot-rev-014**: Line 104: Arbitrary KLE slots.
    - **Deficiency**: Assigns slots 0-8 as Prime blindly.
    - **Recommendation**: Distance-based ranking.
- [x] **Task-prot-rev-022**: Safe regex handling for KLE label extraction.
    - **Deficiency**: Raw KLE labels may contain HTML tags or other formatting clutter.
    - **Recommendation**: Implement safe regex sanitization.

## libs/keyforge-model/src/layout.rs
- [ ] **Task-prot-rev-015**: Line 52: Undocumented duplicates.
    - **Deficiency**: `TryFrom` allows dupes without defined behavior.
    - **Recommendation**: Document resolution policy.

## libs/keyforge-model/src/keyboard.rs
- [ ] **Task-prot-rev-016**: Line 34: Immutable cache.
    - **Deficiency**: `spatial_cache` not updateable.
    - **Recommendation**: `refresh()` method.
- [ ] **Task-prot-rev-017**: Line 98: Unreliable origin fallback.
    - **Deficiency**: Falls back to (0,0) if hand has no keys.
    - **Recommendation**: Validation error.

## libs/keyforge-model/src/corpus.rs
- [x] **Task-prot-rev-018**: Line 80: Inefficient merging.
    - **Deficiency**: Iterates 65536 times.
    - **Recommendation**: Sparse tracking.

## libs/keyforge-model/src/config/definitions.rs
- [ ] **Task-prot-rev-019**: Line 75: ASCII critical bigrams.
    - **Deficiency**: `b.len() == 2` check fails for UTF-8.
    - **Recommendation**: Code-based resolution.

## libs/keyforge-model/src/config/source.rs
- [ ] **Task-prot-rev-020**: Line 130: Single-variant enum.
    - **Deficiency**: `CostMatrixSource::Predefined` only.
    - **Recommendation**: Simplify or expand.

## libs/keyforge-model/bindings/ (TypeScript Generation)

- [x] **Task-prot-rev-021**: Precision loss in `Score` binding.
    - **Deficiency**: `Score` (i64) is mapped to TS `number`. Large score values (scaled by $10^6$) will exceed `Number.MAX_SAFE_INTEGER` ($2^{53}-1 \approx 9 \times 10^{15}$), causing loss of precision in the UI.
    - **Recommendation**: Use `bigint` for `Score` in TS or use string-based representation for the wire format.