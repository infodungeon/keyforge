# Artifact 4: DevEx (Friction) Analysis

**Goal:** Quantify the pain of working on the code.
**Input Data Sources:** `Cargo.toml`.

## 1. Onboarding & Setup Stopwatch

| Workflow Step | Time Taken | Success Rate | Friction Point | Automation Strategy |
| :--- | :--- | :--- | :--- | :--- |
| Build (Rust) | Medium | High | **Compilation** | No `sccache` configured in `Cargo.toml` or `Justfile`. Add for speed. |
| UI Dev | Fast | High | `vite` | HMR is working. |

## 2. Test Suite Health

| Test Type | Duration | Reliability | Mocking Level | Value |
| :--- | :--- | :--- | :--- | :--- |
| Physics Tests | Fast | High | Unit | `compute.rs` has tests, but lacks tests for `u16_to_char` edge cases. |
| Evolution Tests | Fast | High | Unit | `evolution/lib.rs` tests legacy and new entry points. Coverage looks decent. |

## 3. Deployment Pipeline

| Stage | Duration | Failure Rate | Manual Steps? | Fix |
| :--- | :--- | :--- | :--- | :--- |
| Migrations | Variable | Low | Fallback Logic | `jobs.rs` (Line 460) has fallback logic for missing columns. Ensure migrations are strictly applied. |

## Remediation Logic / Rules
1.  **IF** `sccache` missing **THEN** Task: *"[DevEx] Add sccache support to `Justfile`."*
2.  **IF** Legacy Migration Fallbacks exist **THEN** Task: *"[Cleanup] Remove `is_undefined_column` check in `jobs.rs` after verifying prod schema."*