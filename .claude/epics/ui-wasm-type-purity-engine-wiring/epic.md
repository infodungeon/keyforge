---
name: ui-wasm-type-purity-engine-wiring
status: open
created: '2026-02-06T16:20:00.000Z'
updated: '2026-02-06T16:21:00.000Z'
progress: 0
totalTasks: 8
completedTasks: 0
---

## Overview
Converted from PRD: ui-wasm-type-purity-engine-wiring

## Technical Approach
We will bridge the 'Vibe Gap' between WASM and UI.

1. WASM Hardening: Expose the `OptimizationEngine` state via `keyforge-wasm`.
2. Contract Sync: Use the shared `keyforge-protocol` and `keyforge-protocol-bindings` to ensure UI and WASM speak the same language.
3. Logic Wiring: Replace the `setTimeout` loop in `worker.ts` with a real `requestAnimationFrame` or high-frequency polling of the WASM engine.
4. Purity Enforcement: Use the Structural Oracle to define all message types between UI and Worker.

## User Stories (ETS-100x)

### [UI-WASM-01] Expose Engine Evolution Function in WASM
- **Status:** TODO
- **Description:** Implement `evolve` in `keyforge-wasm` to advance simulation state.
- **Acceptance Criteria:** `evolve` exposed via `wasm-bindgen`.

### [UI-WASM-02] Expose Training Update Function in WASM
- **Status:** TODO
- **Description:** Implement `get_training_update` to return DTO with progress metrics.
- **Acceptance Criteria:** `get_training_update` returns correctly typed DTO.

### [UI-WASM-03] Generate TypeScript Types for Engine DTOs
- **Status:** TODO
- **Description:** Configure auto-generation of TS types from Rust DTOs.
- **Acceptance Criteria:** Accurate TS types available for all engine DTOs.

### [UI-WASM-04] Integrate WASM evolve into Worker
- **Status:** TODO
- **Description:** Refactor `worker.ts` to call real `evolve` instead of placeholder logic.
- **Acceptance Criteria:** Worker advances engine state via WASM.

### [UI-WASM-05] Integrate WASM get_training_update into Worker
- **Status:** TODO
- **Description:** Worker periodically polls `get_training_update` and posts to main thread.
- **Acceptance Criteria:** UI receives real-time progress metrics.

### [UI-WASM-06] Achieve Type Purity in worker.ts
- **Status:** TODO
- **Description:** Replace all `any` types in `worker.ts` with generated concrete types.
- **Acceptance Criteria:** Zero `any` types in engine-related code.

### [UI-WASM-07] Validate UI Progress Reporting Accuracy
- **Status:** TODO
- **Description:** Verify that UI accurately reflects real engine state.
- **Acceptance Criteria:** Displayed values match engine state bit-for-bit.

### [UI-WASM-08] End-to-End UI Control of Engine State
- **Status:** TODO
- **Description:** Wire UI controls (Start/Stop/Reset) to real engine operations.
- **Acceptance Criteria:** UI successfully controls underlying simulation.

## Dependencies
- keyforge-wasm (WASM bindings)
- keyforge-ui (TypeScript/Worker)
- wasm-bindgen (Type generation)

## Success Criteria
Inherited from PRD

---
*Generated from PRD by GeminiAutoPM MCP Server*
*Original PRD: .claude/prds/ui-wasm-type-purity-engine-wiring.md*
