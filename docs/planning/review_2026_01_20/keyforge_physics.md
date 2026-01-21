# Review: keyforge-physics

**Date:** 2026-01-20
**Version:** 0.9.0
**Tier:** 1 (The Nucleus)

## libs/keyforge-physics/src/lib.rs
- [x] **Task-phys-rev-001**: Line 114, 130, 153: Hardcoded `KeyCode(0)` as default fill.
    - **Deficiency**: Using a literal `0` is brittle. If `0` is mapped in the corpus, the default layout has an incorrect score.
    - **Recommendation**: Use a named constant or registry-aware default.
- [x] **Task-phys-rev-002**: Line 88: `pinned_keys` ignored in `EngineRequest`.
    - **Deficiency**: Field exists but is ignored by one-off helpers (`score`, `analyze`), leading to confusing behavior.
    - **Recommendation**: Implement logic or remove field.
- [x] **Task-phys-rev-003**: Line 108: `OptimizationResult` misnomer.
    - **Deficiency**: Returned by static `score()` function, implying optimization occurred.
    - **Recommendation**: Return a simpler `ScoringResult` type.

## libs/keyforge-physics/src/error.rs
- [x] **Task-phys-rev-004**: Line 38: `impl From<String> for PhysicsError`.
    - **Deficiency**: Stringly-typed error handling bypasses semantic categorization.
    - **Recommendation**: Remove implementation.

## libs/keyforge-physics/src/verify.rs
- [x] **Task-phys-rev-005**: Line 144, 194: Hardcoded `100.0` fallback cost.
    - **Deficiency**: Magic number for missing keys masks configuration errors.
    - **Recommendation**: Return error or use configurable penalty.
- [x] **Task-phys-rev-006**: Line 234: `dist_raw as i64` truncation.
    - **Deficiency**: Truncation instead of rounding introduces precision errors.
    - **Recommendation**: Use `.round() as i64`.
- [x] **Task-phys-rev-007**: Line 318: Hardcoded scaling factor.
    - **Deficiency**: Duplicates `SCORE_SCALE`.
    - **Recommendation**: Use shared constant.

## libs/keyforge-physics/src/kernel/mechanics.rs
- [x] **Task-phys-rev-008**: Line 44: `dist_raw as i64` (Duplicate).
    - **Deficiency**: Floating point truncation.
    - **Recommendation**: Use `.round() as i64`.
- [x] **Task-phys-rev-009**: Line 73-85: Redundant error mapping.
    - **Deficiency**: Boilerplate `Score::from_f32` checks in hot path.
    - **Recommendation**: Validate weights at compile time.
- [x] **Task-phys-rev-010**: Line 52: Undocumented `odx * odx * t_lat` logic.
    - **Deficiency**: Implicit parabolic effort curve.
    - **Recommendation**: Centralize/Document formula.

## libs/keyforge-physics/src/kernel/compiler.rs
- [ ] **Task-phys-rev-011**: Line 52: `char_freqs` index assumption.
- [x] **Task-phys-rev-012**: Line 104: Monolithic `EngineContext`.
    - **Deficiency**: 30+ fields, hard to maintain.
    - **Recommendation**: Group into sub-structs.
- [x] **Task-phys-rev-013**: Line 131: Deep cloning of corpus data.
    - **Deficiency**: `clone()` on bigrams/trigrams is expensive.
    - **Recommendation**: Use `Arc`.

## libs/keyforge-physics/src/kernel/types.rs
- [x] **Task-phys-rev-014**: Line 24: `ValidatedLayout` checks only underflow.
    - **Deficiency**: Allows overflow and duplicates.
    - **Recommendation**: strict size and uniqueness checks.

## libs/keyforge-physics/src/kernel/compute/scoring.rs
- [ ] **Task-phys-rev-015**: Line 44: Silent skip of missing keys.
- [ ] **Task-phys-rev-016**: Line 52: `Score(i64::MAX)` sentinel.
- [ ] **Task-phys-rev-017**: Line 100: Triple nested loops.

## libs/keyforge-physics/src/kernel/compute/flow.rs
- [ ] **Task-phys-rev-018**: Line 26: Inward-only roll bonus.
- [ ] **Task-phys-rev-019**: Line 62: Duplicate triple-loop logic.

## libs/keyforge-physics/src/kernel/compute/delta.rs
- [ ] **Task-phys-rev-020**: Line 22: Silent out-of-bounds swap.
- [x] **Task-phys-rev-021**: Line 32: Unchecked indexing.
    - **Deficiency**: `u16` to `usize` without bounds check.
    - **Recommendation**: Safe access wrappers.
- [x] **Task-phys-rev-022**: Function complexity.
    - **Deficiency**: 300+ lines of nested logic.
    - **Recommendation**: Refactor into components.

## libs/keyforge-physics/src/kernel/stages/geometry.rs
- [ ] **Task-phys-rev-023**: Line 46: Inconsistent Euclidean distance.
- [ ] **Task-phys-rev-024**: Line 54: `O(N^2)` allocation.

## libs/keyforge-physics/src/kernel/stages/costs.rs
- [ ] **Task-phys-rev-025**: Line 34: Hardcoded `model_key`.
- [ ] **Task-phys-rev-026**: Line 88: Hardcoded zone heuristic.
- [ ] **Task-phys-rev-027**: Line 131: `warn!` in compilation.

## libs/keyforge-physics/src/kernel/stages/corpus.rs
- [ ] **Task-phys-rev-028**: Line 81: Sparse `starts` vectors (65537 size).
- [ ] **Task-phys-rev-029**: Line 138: `prune_trigrams` sort overhead.
- [ ] **Task-phys-rev-030**: Line 42: Deep clone of trigrams.

## libs/keyforge-physics/src/engines/intel_comet_lake.rs
- [x] **Task-phys-rev-031**: Line 54: Heap alloc in `score()`.
    - **Deficiency**: `Box::new(PhysicsScratch)`.
    - **Recommendation**: Caller-provided scratch.
- [ ] **Task-phys-rev-032**: Line 140: Fake AVX2.
- [ ] **Task-phys-rev-033**: Line 144: Unused config.

## libs/keyforge-physics/src/analysis/fingerprint.rs
- [ ] **Task-phys-rev-034**: Line 42: Hardcoded standard strings.
- [ ] **Task-phys-rev-035**: Line 83: Magic threshold 0.2.

## libs/keyforge-physics/src/analysis/heuristics.rs
- [ ] **Task-phys-rev-036**: Line 31: Scratch alloc in loop.
- [ ] **Task-phys-rev-037**: Line 84: Truncate to 5.

## libs/keyforge-physics/src/engines/generic.rs
- [x] **Task-phys-rev-038**: Line 40: Scratch alloc (Duplicate).
    - **Deficiency**: `Box::new`.
    - **Recommendation**: Pool.
- [x] **Task-phys-rev-039**: Line 66: Ignored `pos_map`.
    - **Deficiency**: Rebuilds map from scratch, ignoring optimization.
    - **Recommendation**: Use provided map.

## libs/keyforge-physics/src/kernel/mod.rs
- [x] **Task-phys-rev-040**: Line 30: Inconsistent visibility.
    - **Deficiency**: `fingers` is `pub`, others `pub(crate)`.
    - **Recommendation**: Standardize.