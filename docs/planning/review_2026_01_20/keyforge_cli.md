# Review: keyforge-cli

**Date:** 2026-01-20

## apps/keyforge-cli/src/cmd/init.rs
- [ ] **Task-clii-rev-001**: Line 31: Hardcoded `localhost`.
    - **Deficiency**: Default asset URL.
    - **Recommendation**: Derive from Hive URL.

## apps/keyforge-cli/src/cmd/search.rs
- [x] **Task-clii-rev-002**: Line 62: Hardcoded `.json`.
    - **Deficiency**: Ignores binary assets.
    - **Recommendation**: Use constant.
- [ ] **Task-clii-rev-003**: Line 75: Misleading progress bar.
    - **Deficiency**: Time-based bar for step-based proc.
    - **Recommendation**: Step-based or spinner.