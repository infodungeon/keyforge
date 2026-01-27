# KeyForge 100x Task Workflow

**Version:** 6.0.0
**Role:** Systemic Architect
**Enforcement:** Mandatory

## 1. Core Philosophy

Engineering Truth is achieved through **Systemic Invariants**, not checklists. We do not patch instances; we solve classes of problems.

## 2. The 100x Toolchain

### Search & Discovery
*   **search_file_content (ripgrep):** Primary tool for textual search.
    *   **Rule:** NEVER search the root (`./`) or large directories (e.g., `target/`, `node_modules/`) without strict `include` filters. Root searches flood context with irrelevant metadata.
*   **ast-grep (sg):** Preferred tool for structural code analysis.
    *   **Rule:** Use for identifying patterns across crates (e.g., finding all `unwrap()` calls).
*   **glob:** Use for finding files by pattern before reading.
*   **list_directory:** Use for mapping structural layout.

### Analysis & Strategy
*   **read_file:** Mandatory before any modification.
    *   **Rule:** Always verify the actual content of a file before proposing a `replace` or `write_file`.
*   **delegate_to_agent (codebase_investigator):** Use for mapping complex dependencies or when the impact radius of a change is unknown.

### Modification & Persistence
*   **write_file:** Preferred tool for file creation or total content updates.
    *   **Rule:** Ensures file integrity and "Semantic Truth".
*   **replace:** Use for precise, localized string updates.
    *   **Rule:** Must include sufficient context (3 lines before/after) to ensure uniqueness.
*   **run_shell_command:** Use for git operations, builds, and tests.
    *   **Rule:** Always explain the intent of a modifying shell command before execution.

### Verification & Quality
*   **cargo (check, clippy, test, fmt):** The definitive truth for Rust integrity.
*   **npm/vite:** Truth for UI/Frontend integrity.

---

## 3. The Workflow Protocol

### Phase 1: The Compliance Audit (Mandatory Entry)

#### Executable Actions
1.  **State Audit:** Execute `git_context.sh` (or equivalent shell commands) to inspect the working directory and background processes.
2.  **Issue Synchronization:** Identify the existing GitHub Issue or create a new one using `issue_write` or `run_shell_command`.

#### Criteria for Completion
1.  **Clean State:** Working directory must be clean.
2.  **No Detached Heads:** git HEAD must be attached to a valid branch.
3.  **Issue Exists:** No code execution is permitted without a corresponding GitHub Issue.
4.  **Law Compliance:** Adherence to Architectural Constraints (ARCH-001..ARCH-006).

#### Description
*   **ARCH-001 (UI Purity):** No Data Transformation in UI Components.
*   **ARCH-002 (Thin Controllers):** Controllers delegate to Domain.
*   **ARCH-003 (Deterministic Physics):** No Floating Point Accumulators.
*   **ARCH-004 (Safe SQL):** No Raw SQL; use `sqlx::query!`.
*   **ARCH-005 (Hexagonal Purity):** No IO in kernels.
*   **ARCH-006 (Structural Oracle):** Macros/Traits are the sole source of repetition.
*   **TYPE-003 (Panic-Free):** No `unwrap`/`expect` in production.

---

### Phase 2: The Strategy (Plan)

#### Executable Actions
1.  **Codebase Investigation:** Run `ast-grep` or `codebase_investigator`.
2.  **Zero-Trust Verification:** Explicitly verify file existence and content using `read_file`.
3.  **Logic Brainstorming:** Invoke `delegate_to_agent` (GitHub Copilot MCP).
4.  **Security Scan:** Perform Taint Analysis and LLM Safety checks.
5.  **Plan Proposal:** Present Requirements, Design, and Atomic Steps.

#### Criteria for Completion
1.  **Verified Context:** Plan based on `read_file` output, not assumptions.
2.  **Security Clearance:** No unchecked PII sinks.
3.  **Leverage Alignment:** Explicit use of Type States, Newtypes, or Macros.

---

### Phase 3: The Atomic Transition (Implementation)

#### Executable Actions
1.  **Contract Update:** Update protocols, traits, or docs.
2.  **Type Stubbing:** Define shape before logic.
3.  **Test Creation:** Write tests using `#[keyforge_testing_macros::kf_test]`.
4.  **Implementation Execution:** Use `write_file` (Preferred) or `replace`.
5.  **Pivot (Refactor):** **STOP** if change >3 files or repetitive. Pivot to a macro.

#### Criteria for Completion
1.  **Contract First:** Docs/Contracts precede Logic.
2.  **Test-Driven:** New functionality is verified by tests.
3.  **No Vibe-Patching:** Solutions are systemic.

---

### Phase 4: The Integrity Lock (Verification)

#### Executable Actions
1.  **Integrity Check:** `cargo check` -> `clippy` -> `fmt` -> `test`.
2.  **Security Verification:** The "So What?" Test.
3.  **Debug Protocol (Conditional):** Activate `DEBUGGING_PROTOCOL.md` if 2 failures occur.
4.  **Documentation Update:** Update User, Design, and Ops docs.

#### Criteria for Completion
1.  **Zero Errors/Warnings:** No suppression allowed.
2.  **Two-Strike Limit:** Mandatory Debug Protocol after 2 failures.
3.  **Docs Synced:** Implementation matches documentation.

---

### Phase 5: The Systemic Close (Finalization)

#### Executable Actions
1.  **Continuous Improvement Audit:** Review the work against the CI Checklist (Prevention, Failure Modes, Impact).
2.  **Toolchain Review:** Identify broken, inefficient, or unavailable tools (e.g., MCP server failures) encountered during the task.
3.  **100x Evolution:**
    *   Codify new patterns into skills.
    *   Update behavior constraints (Persona).
    *   **Update Workflow:** **NEVER OVERWRITE.** Create a new versioned file.
    *   Save critical facts to memory.
4.  **Commit:** Semantic commit focusing on *why*.
5.  **Issue Update:** Close/Update the GitHub Issue.
6.  **Push:** Push only upon explicit request.

#### Criteria for Completion
1.  **Systemic Prevention:** Class of problem solved.
2.  **Toolchain Audited:** Failures reported to the system owner for remediation.
3.  **Evolution Recorded:** Learnings are captured in the system (Docs/Memory).
4.  **Immutable History:** Workflow documents are versioned, not edited.
