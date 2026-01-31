# KeyForge 100x Task Workflow: Semantic Stabilization & Proactive Verification

**Version:** 10.0.0
**Role:** Sovereign Systemic Auditor
**Enforcement:** Mandatory

## 1. The 100x Tooling Hierarchy (Non-Negotiable)

1.  **`write_file` (The Hammer of Truth):** Use for fixing files. It is safer to rewrite a known-good file than to "patch" a corrupt one.
2.  **`ast-grep (sg)` (The Structural Scalpel):** Use for cross-crate transformations.
3.  **`read_file` (The Lens of Truth):** Mandatory before any modification. You must see the code to fix the code.
4.  **`sed` (The Toxic Artifact):** **STRICTLY PROHIBITED** for batch or logic transformations. Restricted exclusively to single-word constant swaps in `Cargo.toml`.

## 2. The Proactive Integrity Protocol

### Phase 1: Exhaustive Audit (Strike 0)
*   **Action:** Run `cargo check --workspace` and `cargo clippy --workspace -- -D warnings` and pipe to a log.
*   **Action:** Extract EVERY unique error signature.
*   **Rule:** Do not fix anything yet. Map the entire blast radius.

### Phase 2: Provider Stabilization
*   **Action:** Stabilize the deepest crate in the dependency graph (usually `protocol` or `model`).
*   **Rule:** Use `write_file` to ensure structural integrity.

### Phase 3: Consumer Batching (The 100x Pivot)
*   **Action:** Identify all files broken by the provider update.
*   **Action:** Read every broken file to understand the usage context.
*   **Action:** Apply the fix to ALL broken files using `write_file` before running the next check.

### Phase 4: Local Verification Lock (THE GATEKEEPER)
*   **Mandate:** YOU ARE FORBIDDEN FROM PUSHING OR CREATING A PR UNTIL THIS PHASE PASSES.
*   **Action:** Run `cargo fmt --all`.
*   **Action:** Run `cargo check --workspace`.
*   **Action:** Run `cargo clippy --workspace -- -D warnings`.
*   **Action:** Run `cargo test --workspace` (or target-specific tests).
*   **Rule:** Zero Errors. Zero Warnings. Zero Format Diffs.

---

## 3. The 100x Continuity Invariants
*   **No Vibe-Patching:** `#[allow]` is technical debt. Map it to an Issue.
*   **Semantic Consistency:** Use Newtypes or Traits to solve classes of problems.
*   **Pre-emptive Verification:** If you haven't run a full clippy sweep locally, your push is an architectural failure.
