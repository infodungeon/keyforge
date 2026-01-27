# KeyForge 100x Task Workflow: Efficiency & Structural Integrity

**Version:** 8.0.0
**Role:** Sovereign Systemic Auditor
**Enforcement:** Mandatory

## 1. Core Philosophy

Engineering Truth is achieved through **Structural Invariants**. Velocity is achieved through **Intelligent Batching**. We do not "hack" at warnings; we resolve them systemically.

## 2. The 100x Toolchain Principles

### Tooling Purity Rule (MANDATORY)
*   **write_file:** Primary tool for all structural changes, imports, and multi-line logic. Ensures "Semantic Truth" by overwriting with a verified state.
*   **ast-grep (sg):** Mandatory for cross-crate structural shifts. Use to apply patterns across the workspace.
*   **replace:** Use for surgical, context-aware updates where 3+ lines of context uniquely identify the target.
*   **sed:** **RESTRICTED.** Forbidden for logic or multi-line blocks. Reserved strictly for single-token constant swaps or version bumps. Blind batch `sed` is an architectural failure.

### Efficiency-Quality Invariant
*   **Intelligent Batching:** When presented with multiple errors or warnings, make a reasonable attempt to resolve **ALL** known issues across the workspace before triggering a long-running verification cycle (`clippy`/`test`). 
*   **Non-Destructive Remediation:** I am strictly forbidden from "vibe-patching" by adding `#[allow]` attributes or suppressing lints unless the debt is formally tracked in a GitHub Issue. Unused code should be deleted or integrated, not masked.

---

## 3. The Workflow Protocol

### Phase 1: Compliance & Debt Audit (Strike 0)
1.  **State Audit:** Run `git_context.sh`.
2.  **Issue Linkage:** Comment on the GitHub Issue with the exact error signature and plan.
3.  **Debt Check:** Identify any `#[allow]` or `todo!` markers introduced in previous turns.

### Phase 2: Structural Strategy (Plan)
1.  **Identify the Master Pivot:** Find the 10% of code that solves 90% of the breakage.
2.  **Verify Definitions:** Run `read_file` on struct/enum definitions before usage. Do not guess API visibility.
3.  **Plan Proposal:** Present Atomic Steps focusing on batching.

### Phase 3: Systemic Implementation (Batching)
1.  **Contract Update:** Fix protocols/traits first.
2.  **Multi-File Remediation:** Apply fixes to all crates identified in the previous check.
3.  **No Vibe-Patching:** Use `write_file` to ensure integrity.

### Phase 4: Integrity Lock (Verification)
1.  **Full Cycle:** `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
2.  **Zero-Tolerance:** No warnings or formatting errors allowed at completion.
3.  **Two-Strike Rule:** Pivot to the Diagnostic Protocol (v7) if 2 failures occur.

### Phase 5: Systemic Close
1.  **Technical Debt Sync:** Any introduced `#[allow]` must be migrated to a GitHub Issue.
2.  **Learning Integration:** Codify new invariants into the manifesto.
3.  **Closing Comment:** Update the Issue with final state and closable status.
