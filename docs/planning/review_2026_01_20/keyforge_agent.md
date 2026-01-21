# Review: keyforge-agent

**Date:** 2026-01-20

## apps/keyforge-agent/src/agent/compute.rs
- [x] **Task-agen-rev-001**: Line 60: Unbounded wait.
    - **Deficiency**: Semaphore wait indefinite.
    - **Recommendation**: Timeout.

## apps/keyforge-agent/src/agent/telemetry.rs
- [ ] **Task-agen-rev-002**: Line 48: Hash-based sampling.
    - **Deficiency**: Expensive overhead.
    - **Recommendation**: Modulo sampling.
- [ ] **Task-agen-rev-003**: Line 42: Missing temp.
    - **Deficiency**: Hardcoded 0.0 temperature.
    - **Recommendation**: Pass temperature.

## apps/keyforge-agent/src/agent/network/breaker.rs
- [ ] **Task-agen-rev-004**: Implicit state.
    - **Deficiency**: No Half-Open state.
    - **Recommendation**: Explicit state machine.

## apps/keyforge-agent/src/agent/network/outbox.rs
- [ ] **Task-agen-rev-005**: Line 72: Unscaled recovery.
    - **Deficiency**: Reads all WAL files.
    - **Recommendation**: Streaming.
- [ ] **Task-agen-rev-006**: Line 36: Opaque filenames.
    - **Deficiency**: Nonce only.
    - **Recommendation**: Include IDs.