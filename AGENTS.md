# KeyForge Workspace Constitution

## 1. Systemic Invariants (The Law)
*   **ARCH-005: Hexagonal Purity**: Core crates (`physics`, `evolution`, `model`) MUST have ZERO IO dependencies.
*   **TYPE-003: Panic-Free**: No `unwrap()` or `expect()` in production code. Use `ForgeError`.
*   **SEARCH-001: Surgical Discovery**: Searching the root directory (`./`) is an architectural failure. Use `include` or `dir_path`.
*   **TWO-STRIKE RULE**: After two failed attempts to fix an error, you MUST revert and perform a diagnostic audit.
*   **STRICT SCHEMA**: "Vibe-patching" configuration (e.g., `extra = "allow"` in Pydantic) is FORBIDDEN. All data models must be exhaustive (`extra = "forbid"`).

## 2. Agentic Hierarchy & Skills
| Persona | Activation | Responsibility |
| :--- | :--- | :--- |
| **Consultant** | `sovereign-consultant` | Design, ADRs, C4 Visualization. |
| **Conductor** | (Default) | Issue Locking, PR Orchestration. |
| **Developer** | (Execution) | Atomic `write_file` implementations. |
| **Janitor** | `stabilization-unit` | Ralph Loop mop-up (Mechanical only). |
| **Oracle** | `verification-oracle` | Compliance & Gate-Function check. |

## 3. Mandatory Workflows

### Phase 1: Planning (Consultant)
1.  Activate `sovereign-consultant`.
2.  Run `just plan "Feature"`.
3.  Draft in `.workflow_state/active_plan.md`.
4.  If change affects `>3` crates, generate C4 Container Diagram.

### Phase 2: Implementation (Developer)
1.  Lock Issue in `.workflow_state/active_issue.md`.
2.  **Verify RED**: Run the test/check and document the failure.
3.  Implement fix using `write_file`.

### Phase 3: Stabilization (Janitor)
1.  If mechanical breakage is widespread, activate `stabilization-unit`.
2.  Run `/ralph-loop` until compilation and lints are green.

### Phase 4: Verification (Oracle)
1.  Activate `verification-oracle`.
2.  Provide the **Verification Bundle** (Fresh Check + Parity + Audit).

## 5. Conductor Protocol (Stateful Orchestration)
*   **Source of Truth**: The active track lives in `.agent/CONDUCTOR.json`.
*   **Session Recovery**: Before starting any turn, read `CONDUCTOR.json`. If an `active_issue` is present, resume from the granular plan at `.agent/issues/<ID>/plan.md`.
*   **Track Registry**: All work units must be registered in `.agent/tracks.md`.
*   **Atomic State**: Every track must contain a `spec.md` (What/Why) and a `plan.md` (How/Tasks).

## 6. Automation Commands
*   **Build/Lint:** `just build` / `just lint`
*   **Format:** `just fmt` (Workspace-wide)
*   **Test:** `just test-all` or `cargo test -p <crate>`
*   **Plan:** `just plan` (Initializes state)
