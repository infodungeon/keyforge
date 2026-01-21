# Review: keyforge-hive

**Date:** 2026-01-20

## apps/keyforge-hive/src/services/verification.rs
- [x] **Task-hive-rev-001**: Line 130: Fuzzy Exact check.
    - **Deficiency**: Tolerance applied to Exact engine.
    - **Recommendation**: Exact match required.
- [ ] **Task-hive-rev-002**: Line 115: Repeated parsing.
    - **Deficiency**: Re-parses layout string.
    - **Recommendation**: Cache/Raw.

## apps/keyforge-hive/src/features/submit_result.rs
- [ ] **Task-hive-rev-003**: Line 60: Non-persistent nonce.
    - **Deficiency**: Cache crash resets window.
    - **Recommendation**: Persistent storage requirement.
- [ ] **Task-hive-rev-004**: Line 80: Unbounded queue.
    - **Deficiency**: No backpressure.
    - **Recommendation**: Depth limit.

## apps/keyforge-hive/src/infra/db.rs
- [ ] **Task-hive-rev-005**: Line 135: Global isolation level.
    - **Deficiency**: `REPEATABLE READ` global.
    - **Recommendation**: Transaction-local.
- [x] **Task-hive-rev-006**: Log leakage.
    - **Deficiency**: URL with password logged.
    - **Recommendation**: Redact.

## apps/keyforge-hive/src/features/register_job.rs
- [ ] **Task-hive-rev-007**: Line 130: Late safety check.
    - **Deficiency**: Checks safety after fetch attempt.
    - **Recommendation**: Pre-validate.

## apps/keyforge-hive/src/models.rs
- [ ] **Task-hive-rev-008**: Line 24: Duplicate fields.
    - **Deficiency**: `heatmap` duplicated.
    - **Recommendation**: Use `AnalysisReport`.