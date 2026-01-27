# KeyForge 100x Task Workflow

**Version:** 4.0.0
**Role:** Systemic Architect
**Enforcement:** Mandatory

## 1. Core Philosophy

Engineering Truth is achieved through **Systemic Invariants**, not checklists. We do not patch instances; we solve classes of problems.

## 2. The Workflow Protocol

### Phase 1: The Compliance Audit (Mandatory Entry)

#### Executable Actions
1.  **State Audit:** Execute `git_context.sh` (or equivalent shell commands) to inspect the working directory and background processes.
2.  **Issue Synchronization:** Identify the existing GitHub Issue or create a new one using `issue_write` or `run_shell_command`.

#### Criteria for Completion
1.  **Clean State:** Working directory must be clean (no uncommitted changes unless strictly intentional).
2.  **No Detached Heads:** git HEAD must be attached to a valid branch.
3.  **Issue Exists:** No code execution is permitted without a corresponding GitHub Issue ("No Tickey, No Laundry").
4.  **Law Compliance:** The intended work must strictly adhere to the Architectural Constraints (The Law).

#### Description
*   **ARCH-001 (UI Purity):** No Data Transformation in UI Components (Dumb Views).
*   **ARCH-002 (Thin Controllers):** No "Fat Handlers" in Axum (Delegate to Domain).
*   **ARCH-003 (Deterministic Physics):** No Floating Point Accumulators.
*   **ARCH-004 (Safe SQL):** No Raw SQL; use `sqlx::query!`.
*   **ARCH-005 (Hexagonal Purity):** No IO in kernels.
*   **ARCH-006 (Structural Oracle):** Macros/Traits are the sole source of repetition.
*   **TYPE-003 (Panic-Free):** No `unwrap`/`expect` in production.

---

### Phase 2: The Strategy (Plan)

#### Executable Actions
1.  **Codebase Investigation:** Run `ast-grep` (`sg`) or `codebase_investigator` to map structural dependencies.
2.  **Zero-Trust Verification:** Explicitly verify file existence and content using `read_file` or `grep` (never assume).
3.  **Logic Brainstorming:** Invoke `delegate_to_agent` (GitHub Copilot MCP) for core algorithm verification.
4.  **Security Scan:** Perform Taint Analysis and LLM Safety checks on the proposed approach.
5.  **Plan Proposal:** Present the plan to the user, broken down by Requirement, Design, and Steps.

#### Criteria for Completion
1.  **Verified Context:** Plan is based on verified file contents, not assumptions.
2.  **Security Clearance:** Plan contains no unchecked PII sinks or Prompt Injection vectors.
3.  **Leverage Alignment:** The plan explicitly uses Type States, Newtypes, or Macros where applicable.

#### Description
*   **Requirement Analysis:** Statement of *what* needs to change.
*   **Solution Design:** Description of the architectural approach.
*   **Execution Steps:** Atomic, sequential steps to achieve the goal.
*   **Taint Analysis:** Tracing data from Source (Input) to Sink (Output/Log).

---

### Phase 3: The Atomic Transition (Implementation)

#### Executable Actions
1.  **Contract Update:** Update protocols, traits, or documentation (`.md` files).
2.  **Type Stubbing:** Create empty structs/traits to define the shape before logic.
3.  **Test Creation:** Write tests using `#[keyforge_testing_macros::kf_test]`.
4.  **Implementation Execution:**
    *   Use `write_file` for structural truth (preferred).
    *   Use `replace` only for minor, precise edits.
    *   Use `ast-grep` for multi-file pattern transformations.
5.  **Pivot (Refactor):** **STOP** if the change touches >3 files or requires repetitive boilerplate. Pivot to a macro solution.

#### Criteria for Completion
1.  **Contract First:** Documentation/Contracts must exist before Logic.
2.  **Test-Driven:** Tests must exist for the new functionality.
3.  **100x Rules Compliance:**
    *   **Type States:** Invalid states are unrepresentable.
    *   **Newtypes:** Primitives are wrapped in domain types.
    *   **Macros:** Boilerplate is generated, not written.
4.  **Domain Axiom Compliance:**
    *   **Finger 0:** Mapped to Thumb.
    *   **Determinism:** No platform-specific float usage.
5.  **No Vibe-Patching:** Implementation solves the *class* of problem, not just the instance.

#### Description
*   **Friction Trigger:** The condition where a change touches >3 files or requires repetitive typing, signaling the need for abstraction.
*   **Vibe-Patching:** Tweaking code without understanding the structural pattern.

---

### Phase 4: The Integrity Lock (Verification)

#### Executable Actions
1.  **Compilation:** Run `cargo check`.
2.  **Linting:** Run `cargo clippy`.
3.  **Formatting:** Run `cargo fmt`.
4.  **Testing:** Run `cargo test`.
5.  **Security Verification:** Evaluate the actual security impact of the changes.
6.  **Debug Protocol (Conditional):** If 2 fix attempts fail:
    *   **Stop Coding.**
    *   Read `docs/engineering/DEBUGGING_PROTOCOL.md`.
    *   Instrument Input/Transform/Output states.
7.  **Documentation Update:** Update User, Design, and Ops docs.

#### Criteria for Completion
1.  **Zero Errors:** Code compiles without errors.
2.  **Zero Warnings:** No Clippy warnings remain (suppression is forbidden).
3.  **Tests Pass:** All tests pass.
4.  **Two-Strike Limit:** No more than 2 failed fix attempts without entering Debug Mode.
5.  **Impact Verified:** The "So What?" test confirms the security/privacy stance is secure.
6.  **Docs Synced:** Documentation matches the implementation.

#### Description
*   **The "So What?" Test:** A mental check to distinguish theoretical vulnerabilities from actual risks.
*   **Isolation Rule:** During debug, modify *instrumentation only*, not logic.

---

### Phase 5: The Systemic Close (Finalization)

#### Executable Actions
1.  **Continuous Improvement Audit:** Review the work against the CI Checklist (Prevention, Failure Modes, Impact).
2.  **100x Evolution:**
    *   Codify new patterns into skills.
    *   Update behavior constraints (Persona).
    *   **Update Workflow documents:** **NEVER OVERWRITE.** Create a new version file (e.g., `tasks_workflow_v4.md`).
    *   Save critical facts to memory.
3.  **Commit:** Stage and commit changes with a semantic message.
4.  **Issue Update:** Update the GitHub Issue with the final status.
5.  **Push:** Push changes to remote (if requested).

#### Criteria for Completion
1.  **Systemic Prevention:** The solution solves the *class* of problem, not just the instance.
2.  **Audit Complete:** Tool failure modes and impact radius have been considered.
3.  **Evolution Recorded:** Learnings are captured in the system (Docs/Memory).
4.  **Immutable History:** Workflow documents are strictly versioned, not edited in place.

#### Description
*   **Semantic Commit:** A commit message focusing on the *why* and *intent*, not just the file changes.
