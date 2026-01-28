# KeyForge 100x Task Workflow: Semantic Stabilization

**Version:** 9.1.0
**Role:** Sovereign Systemic Auditor
**Enforcement:** Mandatory (Refer to AGENTS.md for Systemic Invariants)

## 1. The 100x Tooling Hierarchy
1.  **`write_file`**: Primary implementation tool.
2.  **`ast-grep (sg)`**: Cross-crate structural transformations.
3.  **`read_file`**: Mandatory before modification.
4.  **`ralph-loop`**: Restricted to **Stabilization Unit** (Mop-up).

## 2. The Semantic Stabilization Protocol

### Phase 1: Exhaustive Audit (Verify RED)
*   **Action:** Run a full workspace check.
*   **Action:** Extract the failure signature.
*   **Mandate:** Do not fix until the failure is documented.

### Phase 2: Provider Stabilization
*   **Action:** Stabilize the deepest crate (Protocol/Model).
*   **Rule (TWO-STRIKE)**: If the fix fails twice, revert to Phase 1.

### Phase 3: Consumer Batching
*   **Action:** Identify and read all consumer breakage.
*   **Action:** Apply mechanical fixes. If `>20` files, activate `stabilization-unit`.

### Phase 4: Integrity Lock (Gate Function)
*   **Action:** Run verification (`check`, `clippy`, `fmt`, `test`).
*   **Requirement:** Produce a **Verification Bundle** per `Verification Oracle` skill.

---

## 3. Continuity Invariants
*   **No Vibe-Patching**: No `#[allow]`, no `extra = "allow"`.
*   **Semantic Consistency**: Use Traits/Newtypes for structural leverage.
*   **Done Definition**: Zero Errors, Zero Warnings, Fmt, Documentation, and Verified Parity.