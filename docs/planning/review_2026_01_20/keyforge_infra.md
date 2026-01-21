# Review: keyforge-infra

**Date:** 2026-01-20

## libs/keyforge-infra/src/fs/init.rs
- [ ] **Task-infra-rev-001**: Line 31: Hardcoded asset paths.
    - **Deficiency**: `config/keycodes` etc. are string literals.
    - **Recommendation**: Shared constants.
- [ ] **Task-infra-rev-002**: Line 103: Hardcoded extensions.
    - **Deficiency**: Only checks `.mpk.zst` / `.json`.
    - **Recommendation**: Registry.

## libs/keyforge-infra/src/fs/paths.rs
- [ ] **Task-infra-rev-003**: Line 42: Weak workspace detection.
    - **Deficiency**: Checks only for `keyboards` dir.
    - **Recommendation**: Marker file.
- [ ] **Task-infra-rev-004**: Line 60: Unchecked Option.
    - **Deficiency**: Always returns Some.
    - **Recommendation**: Return result directly.

## libs/keyforge-infra/src/net/sync.rs
- [x] **Task-infra-rev-005**: Line 100: Fragile bootstrap logic.
    - **Deficiency**: String matching (`contains`) to find essentials.
    - **Recommendation**: Metadata tags.
- [ ] **Task-infra-rev-006**: Line 60: Silent invalid path skip.
    - **Deficiency**: `normalize_path` failure ignored.
    - **Recommendation**: Log error.

## libs/keyforge-infra/src/asset/manager.rs
- [ ] **Task-infra-rev-007**: Line 30: Redundant mapping.
    - **Deficiency**: Duplicates `init.rs` folder logic.
    - **Recommendation**: Centralize.
- [ ] **Task-infra-rev-008**: Line 114: Bundle structure assumption.
    - **Deficiency**: Checks `1grams.mpk.zst` only.
    - **Recommendation**: Manifest check.

## libs/keyforge-infra/src/asset/fs_provider.rs
- [x] **Task-infra-rev-009**: Line 185: Semantic string matching.
    - **Deficiency**: `id.contains("_std")` triggers logic.
    - **Recommendation**: Metadata field.
- [ ] **Task-infra-rev-010**: Line 160: Sequential IO.
    - **Deficiency**: Serial loads for corpus parts.
    - **Recommendation**: Parallel IO.

## libs/keyforge-infra/src/util/common.rs
- [ ] **Task-infra-rev-011**: Line 54: Stubs.
    - **Deficiency**: `generate_cost_profile` is a stub.
    - **Recommendation**: Implement.

## libs/keyforge-infra/src/util/corpus.rs
- [ ] **Task-infra-rev-012**: Line 120: Limited hex support.
    - **Deficiency**: 2-digit hex only.
    - **Recommendation**: 4-digit support.
- [ ] **Task-infra-rev-013**: Line 145: Expensive injection.
    - **Deficiency**: $O(N)$ iteration for synthetic data.
    - **Recommendation**: Optimization.
- [ ] **Task-infra-rev-014**: Line 230: Double sort.
    - **Deficiency**: Sorting at end of injection.
    - **Recommendation**: Invariant maintenance.
