---
name: data-005-content-addressable-jobs
status: open
created: '2026-02-06T16:15:00.000Z'
updated: '2026-02-06T16:16:00.000Z'
progress: 0
totalTasks: 4
completedTasks: 0
---

## Overview
Converted from PRD: data-005-content-addressable-jobs

## Technical Approach
We will transition from 'ID-based Fingerprinting' to 'Content-Addressable Fingerprinting'. 

1. Hash Computation: Compute SHA-256 of the raw asset bytes during the Valkey load operation.
2. Struct Update: Store the 256-bit hash in the `Asset` struct.
3. Fingerprint logic: Modify `calculate_fingerprint` in `common.rs` to include the `content_hash` of every associated asset.
4. Validation: Use `just test-infra` and `just test-compute` to verify that changing content results in a new fingerprint.

## User Stories (ETS-100x)

### [DATA-005-01] Extend Asset Structure for Content Hashing
- **Status:** TODO
- **Description:** Add `content_hash` field to `Asset` struct in `keyforge-infra`.
- **Acceptance Criteria:** `Asset` struct includes `content_hash: [u8; 32]`.

### [DATA-005-02] Compute Content Hash During Asset Loading
- **Status:** TODO
- **Description:** Enhance Valkey asset loader to compute SHA-256 hash of asset content on every load.
- **Acceptance Criteria:** Loaded `Asset` objects have accurate `content_hash` populated.

### [DATA-005-03] Incorporate Content Hash into Fingerprint Calculation
- **Status:** TODO
- **Description:** Refactor `calculate_fingerprint` to use `content_hash` as a core input.
- **Acceptance Criteria:** Identical ID with different content results in different fingerprints.

### [DATA-005-04] Verify Cache Invalidation with content-addressable fingerprints
- **Status:** TODO
- **Description:** Create integration tests demonstrating cache invalidation on content change.
- **Acceptance Criteria:** Tests verify that stale data is NOT served from cache when content updates.

### [DATA-005-05] Deduplicate Fingerprinting Logic (ARCH-006)
- **Status:** TODO
- **Description:** Remove duplicated `calculate_fingerprint` in `keyforge-infra` and consolidate on `calculate_corpora_fingerprint` in `keyforge-model`.
- **Acceptance Criteria:** Zero duplicate hashing logic for corpora.

## Dependencies
- keyforge-infra (Asset loading)
- keyforge-compute (Job ID generation)
- Valkey (Asset storage)

## Success Criteria
Inherited from PRD

---
*Generated from PRD by GeminiAutoPM MCP Server*
*Original PRD: .claude/prds/data-005-content-addressable-jobs.md*
