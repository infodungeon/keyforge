# Artifact 6: Master Remediation Backlog (The SOW)

**Goal:** Synthesized list of actionable tasks ready for Project Management import.
**Source:** Aggregated from deep-dive analysis.

| ID | Task Summary | Description | Category | Priority (P0-P3) | T-Shirt Size (S/M/L) | Assigned Role |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **REM-01** | **[Refactor] Physics Allocations** | Rewrite `u16_to_char` in `compute.rs` to avoid String allocation in analysis path. | Performance | **P1** | S | Rust Backend |
| **REM-02** | **[UI] Optimize Drag Loop** | Refactor `VisualBuilder.tsx` to stop using `updateKeys` (state trigger) on every mouse move. Use local refs + `requestAnimationFrame`. | Performance | **P1** | M | Frontend |
| **REM-03** | **[Database] Simplify Claim Query** | Refactor `jobs.rs` (Line 337) to remove complex `jsonb_build_object` construction. Build JSON in Rust. | Architecture | **P2** | L | Backend Lead |
| **REM-04** | **[Cleanup] Remove Legacy Fallback** | Remove `is_undefined_column` logic in `jobs.rs` (Line 460) to enforce schema consistency. | Tech Debt | **P3** | S | Backend |
| **REM-05** | **[UI] Standardize Constants** | Extract `UNIT` and `SNAP` from `VisualBuilder.tsx` into a `geometry.ts` config file. | Reuse | **P3** | S | Frontend |
| **REM-06** | **[Security] Validate Pinned Keys** | Add schema validation for `pinned_keys` JSON in `jobs.rs` before database insertion. | Security | **P1** | M | Backend |
| **REM-07** | **[Reuse] Extract Grid Component** | Extract the SVG Grid rendering from `VisualBuilder.tsx` to share with `ArenaCanvas.tsx`. | Reuse | **P3** | S | Frontend |
| **REM-08** | **[Refactor] Centralize Config Loading** | Move `load_config_from_standard_paths` from `agent` to `keyforge-infra` to share with CLI. | Reuse | **P2** | M | Rust Backend |

## Priority Definitions
*   **P0 (Critical):** Security vulnerability or Production outage risk.
*   **P1 (High):** Major impediment to scaling or velocity. Fix before next major feature.
*   **P2 (Medium):** Technical Debt that slows down dev.
*   **P3 (Low):** Housekeeping.