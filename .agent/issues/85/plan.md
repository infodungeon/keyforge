# Track #85 Implementation Plan: ADR-021 Infrastructure

## Phase 1: Protocol Stabilization (DTOs)
- [x] Task: Define `UserProfileDto` and `UserPreferencesDto` in `keyforge-protocol`.
- [x] Task: Define `BiometricProfileDto` and `LatencyStatsDto`.
- [x] Task: Implement `From`/`Into` traits for all new types.
- [x] Task: Verify with `cargo check -p keyforge-protocol`.

## Phase 2: Schema Manifestation (Migrations)
- [x] Task: Create SQL migration for `users` and `user_preferences` tables.
- [x] Task: Create SQL migration for `biometric_profiles` and `latencies`.
- [x] Task: Create SQL migration for `layout_submissions`.
- [x] Task: Integrate "Enhanced Token Tracking" (research_metrics) into schema.

## Phase 3: Persistence Implementation (Repositories)
- [x] Task: Implement `UserRepository` in `keyforge-persistence`.
- [x] Task: Implement `BiometricRepository`.
- [x] Task: Implement `CommunityRepository`.
- [x] Task: Implement `ResearchRepository` (Token Tracking).
- [x] Task: Integrate all repositories into `keyforge-hive` AppState.

## Phase 4: Verification
- [x] Task: Run full integration tests for the persistence layer.
- [x] Task: Produce **Verification Bundle**.

# Status: COMPLETED
All ADR-021 entities have been successfully integrated into the Protocol and Persistence layers. The schema has been migrated, and all repositories are verified with live PostgreSQL integration tests.
