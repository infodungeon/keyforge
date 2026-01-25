# KeyForge 100x Task Workflow

**Version:** 2.2.0
**Role:** Systemic Architect
**Enforcement:** Mandatory

## 1. Core Philosophy

Engineering Truth is achieved through **Systemic Invariants**, not checklists. We do not patch instances; we solve classes of problems.

## 2. The Workflow Protocol

### Phase 1: The Compliance Audit (Mandatory Entry)
*Goal: Establish the "Truth" of the current state before any action.*

1.  **State Audit**
    *   Execute `git_context.sh` (or equivalent) to verify a clean working directory.
    *   Check for orphaned background processes.
2.  **Issue Synchronization ("No Tickey, No Laundry")**
    *   **Rule:** No code is written without a corresponding GitHub Issue.
    *   **Action:** Create or Identify the issue. All significant plan updates are logged here.
3.  **The Law Compliance Check (Architectural Constraints)**
    *   **ARCH-001 (UI Purity):** No Data Transformation in UI Components (Dumb Views).
    *   **ARCH-002 (Thin Controllers):** No "Fat Handlers" in Axum (Delegate to Domain).
    *   **ARCH-003 (Deterministic Physics):** No Floating Point Accumulators.
    *   **ARCH-004 (Safe SQL):** No Raw SQL; use `sqlx::query!`.
    *   **ARCH-005 (Hexagonal Purity):** No IO in kernels.
    *   **ARCH-006 (Structural Oracle):** Macros/Traits are the sole source of repetition.
    *   **TYPE-003 (Panic-Free):** No `unwrap`/`expect` in production.

### Phase 2: The Strategy (Plan)
*Goal: Define a clear, logical path to the solution.*

1.  **Codebase Investigation**
    *   Use `ast-grep` (`sg`) or `codebase_investigator` to map structural impact.
    *   **Zero-Trust Context:** Never assume a library/file exists. Verify with `read_file` or `grep`.
2.  **Logic Brainstorming**
    *   Invoke GitHub Copilot MCP for complex logic verification (Core Algorithms).
3.  **Security & Privacy Check**
    *   **Taint Analysis:** Trace PII from Source to Sink.
    *   **LLM Safety:** Check for Prompt Injection risks.
4.  **Plan Proposal**
    *   **Requirement Analysis:** Clearly state *what* needs to change.
    *   **Solution Design:** Describe the architectural approach.
    *   **Execution Steps:** Detailed, atomic steps to achieve the goal.

### Phase 3: The Atomic Transition (Implementation)
*Goal: Systemic implementation using high-precision tools and strict coding rules.*

1.  **Contract First**
    *   Update protocols, traits, or documentation *before* implementation.
    *   **Logic Bootstrapping:** Type-stubs and docs precede logic.
2.  **Test-Driven Oracle**
    *   Write/Update tests using `#[keyforge_testing_macros::kf_test]`.
    *   **Forbidden:** Manual `#[cfg(test)]` or `#[allow]`.
3.  **The 100x Implementation Rules**
    *   **Type States:** Invalid states must be unrepresentable (e.g., `Builder`, `Ready`/`Pending`).
    *   **Newtypes:** Banish primitive obsession (e.g., `struct Meter(f64)` vs `f64`).
    *   **Macros:** Abstract repetitive patterns (e.g., `#[derive(DomainLogic)]`).
4.  **Domain Axioms**
    *   **Finger Identity:** Finger 0 is strictly the **Thumb**.
    *   **Determinism:** Integer arithmetic only. No platform-specific float behavior.
5.  **Implementation Execution**
    *   **Tool Preference:** Prefer `write_file` over `replace` for structural integrity.
    *   **Use `ast-grep`** for multi-file pattern transformations.
    *   **Friction Trigger:** If a change touches >3 files or requires repetitive boilerplate, **STOP**. Refactor (Pivot) to a structural solution (e.g., macro).

### Phase 4: The Integrity Lock (Verification)
*Goal: The "100x Definition of Done".*

1.  **Compilation:** Zero errors (`cargo check`).
2.  **Linting:** Zero warnings (`cargo clippy`). *Fix* warnings; never suppress them.
3.  **Formatting:** Workspace-wide formatting (`cargo fmt`).
4.  **Testing:** All tests pass (`cargo test`).
5.  **Security Audit:**
    *   **The "So What?" Test:** Verify actual security impact.
6.  **Documentation:** Update User, Design, and Ops docs.

### Phase 5: The Systemic Close (Finalization)
*Goal: Lock in the learning.*

1.  **Continuous Improvement Checklist**
    *   Did I create a Systemic Prevention?
    *   Did I audit Tool Failure Modes?
    *   Is the Global Impact Radius contained?
2.  **Commit**
    *   Format: Semantic commit message focusing on *why*.
3.  **Issue Update**
    *   Update GitHub Issue with final status and close if appropriate.
4.  **Push**
    *   Push changes only upon explicit request.
5.  **100x Developer Evolution**
    *   **Add or Update skills**: Codify new patterns into skills.
    *   **Enhance Persona**: Update behavior constraints.
    *   **Update Workflow**: Create new higher version numbered document with Improved workflow.
    *   **Save Facts/Rules**: Save critical facts to memory.

## 3. Critical Invariants

*   **Never "Vibe-Patch":** If a problem cannot be solved systemically, stop and request a refactoring turn.
*   **The "Two-Strike" Rule:** After two failed fix attempts, a mandatory diagnostic turn is required.
    *   **Protocol:** Activate `docs/engineering/DEBUGGING_PROTOCOL.md`.
    *   **The Isolation Rule:** Stop changing code. Modify **instrumentation** only.
    *   **Mandatory Evidence:** You must log and prove **Input State**, **Transformation Logic**, and **Output State** before a third attempt.
