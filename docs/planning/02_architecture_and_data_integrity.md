# Artifact 2: Architecture & Data Integrity Report

**Goal:** Map the structural flaws, data integrity risks, and scalability bottlenecks.
**Input Data Sources:** Code Review (`jobs.rs`, `VisualBuilder.tsx`, `evolution/lib.rs`).

## 1. Integrity & Anti-Pattern Map

| Component / Table | Issue Category | Evidence / Line # | Risk Level | Proposed Fix |
| :--- | :--- | :--- | :--- | :--- |
| `jobs.rs` | Schema Coupling | Line 337 (`jsonb_build_object`) | High | SQL query is tightly coupled to the JSON structure of `JobRequest`. Break into structured columns or handle serialization in Rust. |
| `VisualBuilder.tsx` | Logic Leak | Line 18 (`const UNIT = 54`) | Medium | Physical constants hardcoded in UI. Breaks if we change render scale. |
| `compute.rs` | Performance | Line 148 (Trigram Loop) | Medium | Triple nested loop over candidates. Needs geometric pruning for layouts with many duplicate keys. |
| `evolution/lib.rs` | Error Handling | Line 104 (`unwrap_or_else`) | Low | Panic risk if default layout generation fails (unlikely but possible). |

## 2. Query Performance Audit
*Potential bottlenecks.*

| Endpoint / Job | Avg Duration | Call Count | Root Cause | Optimization |
| :--- | :--- | :--- | :--- | :--- |
| `claim_job` | ~50ms | High | `SKIP LOCKED` + Complex Join | Monitor DB CPU. Consider denormalizing `params_json` to avoid 4-table join on critical path. |

## Remediation Logic / Rules
1.  **IF** SQL constructs JSON **THEN** Task: *"[Refactor] Simplify `claim_job` query to return flat rows."*
2.  **IF** UI has Logic Leaks **THEN** Task: *"[Refactor] Extract `UNIT` and `SNAP` to `apps/keyforge-ui/src/constants.ts`."*