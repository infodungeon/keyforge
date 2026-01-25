# ISSUE-039: Hive Data Mapping Projection Trait (V2)

**Goal**: Systemically eliminate manual, fragile mapping between SQLx/JSON and Model by implementing a `Projection` trait in `keyforge-model`.

**Status**: OPEN
**Phase**: 1. Compliance Audit

## Findings
- Manual mapping in `claim_job` and `get_config` in `apps/keyforge-hive/src/infra/repositories/jobs/core.rs`.
- Fragile error handling (`map_err` boilerplate).
- Violation of ARCH-002 (Fat Handlers/Logic Leakage).

## Plan (100x Filter)
- **Lever 1: Type States**: Use `Projection` to transition safely from DTO to Domain.
- **Lever 2: Newtypes**: Ensure mapping targets domain newtypes (KeyIndex, etc).
- **Lever 3: Macros**: Leverage `sqlx::FromRow` for DTO population.

## Checklist
- [x] Phase 1: Compliance Audit
- [ ] Phase 2: Strategy (Plan)
- [ ] Phase 3: Atomic Transition (Implementation)
- [ ] Phase 4: Integrity Lock (Verification)
- [ ] Phase 5: Systemic Close (Finalization)
