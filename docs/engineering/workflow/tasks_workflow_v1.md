# KeyForge 100x Task Workflow

**Version:** 1.0.0
**Role:** Systemic Architect
**Enforcement:** Mandatory

## 1. Core Philosophy

Engineering Truth is achieved through **Systemic Invariants**, not checklists. We do not patch instances; we solve classes of problems.

## 2. The Workflow Protocol

### Phase 1: The Compliance Audit (Mandatory Entry)
*Goal: Establish the "Truth" of the current state before any action.*

1.  **State Audit**
    *   Execute `git_context.sh` (or equivalent) to verify a clean working directory.
    *   Identify detached heads or dirty contexts immediately.
2.  **Issue Synchronization ("No Tickey, No Laundry")**
    *   **Rule:** No code is written without a corresponding GitHub Issue.
    *   **Action:** Create or Identify the issue. All significant plan updates are logged here.
3.  **The Law Compliance Check**
    *   **ARCH-005 (Hexagonal Purity):** No IO in kernels.
    *   **TYPE-003 (Panic-Free):** No `unwrap`/`expect` in production.
    *   **Determinism:** Integer arithmetic only.
    *   **Structure:** Macros/Traits are the sole source of repetition.

### Phase 2: The Strategy (Plan)
*Goal: Define a "Correct-by-Construction" path.*

1.  **Codebase Investigation**
    *   Use `ast-grep` (`sg`) or `codebase_investigator` to map structural impact.
    *   **Constraint:** Do not assume; verify.
2.  **Logic Brainstorming**
    *   Invoke GitHub Copilot MCP for complex logic verification.
3.  **Plan Proposal**
    *   Present a concise plan prioritizing:
        *   **Type States** over runtime checks.
        *   **Newtypes** over primitives.
        *   **Macros** over boilerplate.

### Phase 3: The Atomic Transition (Implementation)
*Goal: Systemic implementation using high-precision tools.*

1.  **Contract First**
    *   Update protocols, traits, or documentation *before* implementation.
2.  **Test-Driven Oracle**
    *   Write/Update tests using `#[keyforge_testing_macros::kf_test]`.
    *   **Forbidden:** Manual `#[cfg(test)]` or `#[allow]`.
3.  **Implementation**
    *   Prefer `write_file` for structural truth.
    *   Use `ast-grep` for multi-file pattern transformations.
    *   **Friction Trigger:** If a change touches >3 files or requires repetitive boilerplate, **STOP**. Refactor (Pivot) to a structural solution (e.g., macro).

### Phase 4: The Integrity Lock (Verification)
*Goal: The "100x Definition of Done".*

1.  **Compilation:** Zero errors (`cargo check`).
2.  **Linting:** Zero warnings (`cargo clippy`). *Fix* warnings; never suppress them.
3.  **Formatting:** Workspace-wide formatting (`cargo fmt`).
4.  **Testing:** All tests pass (`cargo test`).
5.  **Documentation:** Update User, Design, and Ops docs.

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

## 3. Critical Invariants

*   **Never "Vibe-Patch":** If a problem cannot be solved systemically, stop and request a refactoring turn.
*   **The "Two-Strike" Rule:** After two failed fix attempts, a mandatory diagnostic turn (instrumentation) is required.
