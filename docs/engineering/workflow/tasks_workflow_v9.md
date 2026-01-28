# KeyForge 100x Task Workflow: Semantic Stabilization

**Version:** 9.0.0
**Role:** Sovereign Systemic Auditor
**Enforcement:** Mandatory

## 1. The 100x Tooling Hierarchy (Non-Negotiable)

1.  **`write_file` (The Hammer of Truth):** Use for fixing files. It is safer to rewrite a known-good file than to "patch" a corrupt one.
2.  **`ast-grep (sg)` (The Structural Scalpel):** Use for cross-crate transformations.
3.  **`read_file` (The Lens of Truth):** Mandatory before any modification. You must see the code to fix the code.
4.  **`sed` (The Toxic Artifact):** Restricted to single-word constant swaps in `Cargo.toml`. Usage for logic is a strike.
5.  **`SEARCH-001` (Targeted Exploration):** Searching the root directory (`./`) is an architectural failure. Use `include` or `dir_path` for every operation. Enforcement is automated via `ops/scripts/context_safeguard.py`.
6.  **`CONTEXT-001` (Structural Minification):** For complex files, use `just context <FILE>` to read headers first. Reading > 500 lines of raw code without minification is a strike.

## 2. The Semantic Stabilization Protocol

### Phase 1: Exhaustive Audit (Strike 0)
*   **Action:** Run a full workspace check and pipe to a log.
*   **Action:** Extract EVERY unique error signature.
*   **Rule:** Do not fix anything yet. Map the entire blast radius.

### Phase 2: Provider Stabilization
*   **Action:** Stabilize the deepest crate in the dependency graph (usually `protocol` or `model`).
*   **Rule:** Use `write_file` to ensure structural integrity.

### Phase 3: Consumer Batching (The 100x Pivot)
*   **Action:** Identify all files broken by the provider update.
*   **Action:** Read every broken file to understand the usage context.
*   **Action:** Apply the fix to ALL broken files using `write_file` before running the next check.

### Phase 4: Integrity Lock
*   **Action:** Run verification (`check`, `clippy`, `fmt`, `test`).
*   **Rule:** If a cycle returns >0 errors, return to Phase 1. Do not "quick-fix".

---

## 3. The 100x Continuity Invariants
*   **No Vibe-Patching:** `#[allow]` is technical debt. Map it to an Issue.
*   **Semantic Consistency:** Use Newtypes or Traits to solve classes of problems.
*   **Zero-Check Drift:** If you find yourself checking more than once every 5 minutes, you are vibe-coding. Stop and audit.
