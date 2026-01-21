# Review: keyforge-persistence

**Date:** 2026-01-20

## libs/keyforge-persistence/src/store/autosave.rs
- [ ] **Task-pers-rev-001**: Line 100: Full file read.
    - **Deficiency**: Reads entire session file into memory.
    - **Recommendation**: Streaming parse.
- [ ] **Task-pers-rev-002**: Line 145: Primitive debounce.
    - **Deficiency**: Time-based only.
    - **Recommendation**: Proper debounce task.

## libs/keyforge-persistence/src/repo/user_repo.rs
- [ ] **Task-pers-rev-003**: Line 130: 100k sample limit.
    - **Deficiency**: Hardcoded cap.
    - **Recommendation**: Constant/Config.
- [ ] **Task-pers-rev-004**: Line 160: Locking overhead.
    - **Deficiency**: Lock per batch.
    - **Recommendation**: WAL/Background writer.
- [ ] **Task-pers-rev-005**: Line 190: Hardcoded profile path.
    - **Deficiency**: `personal_cost.json`.
    - **Recommendation**: Named profiles.

## libs/keyforge-persistence/src/error.rs
- [ ] **Task-pers-rev-006**: String errors.
    - **Deficiency**: `AssetLoad(String)`.
    - **Recommendation**: Structured errors.