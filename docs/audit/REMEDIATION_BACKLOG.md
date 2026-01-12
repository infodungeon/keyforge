# KeyForge Remediation Backlog (Final)

## HIGH: Architecture & Scalability
1.  **Unified Runner Crate**: [DONE] Created `libs/keyforge-runner`. Eliminated duplication between UI and Agent.
2.  **Spatial Cache**: Move `finger_origins` and travel-cost pre-computations into the `Keyboard` model so they are calculated ONCE per definition, not $O(K^2)$ times per job start.
3.  **Hydrated Repositories**: Replace SQL JSONB reconstruction with SQLX struct mapping.
4.  **Registry Synchronization**: Ensure all components (Agent, UI, Hive) use the *exact same* registry file hash for a given job.

## CRITICAL: Security & Data Integrity
5.  **Harden Asset IO**: Replace `self.root.join(path)` with a canonicalized, path-sandboxed implementation in `FsProvider`.
6.  **Verify System Assets**: Force hash verification for "system" assets in `AssetManager`, not just user assets.
7.  **Secure by Construction**: Remove `signature: None` from `ResultSubmission`. Require a signed payload at the point of creation.
8.  **Harden CLI IO**: Stop using temporary files for `JobConfig` pass-through between CLI and Agent; use anonymous pipes or stdin to avoid disk-leakage of job data.


## MEDIUM: Reliability
9.  **Async-Safe WAL**: Refactor `ResultOutbox` to use `tokio::fs` and implement a 100MB/500-file size limit.
10. **Dynamic Service Resolution**: Remove hardcoded "3000->3001" port replacement. Use a proper Discovery service or Service Manifest.
11. **Consistent Verification**: Ensure `VerificationService` uses the *exact* `SearchParams` from the job record, not defaults.
12. **Coordinate Maintenance**: Move Hive cron tasks to a robust job scheduler (e.g. `apalis` or `tokio-cron-scheduler`) to prevent piling.