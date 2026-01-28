# KeyForge Planning Workflow: The Architect's Loop

**Version:** 1.1.0
**Role:** Sovereign Systemic Architect
**Goal:** Reach a "Verified Design State" before a single line of code is written.

## 1. The Planning Cycle
*   **Action:** All planning must be recorded in `.workflow_state/active_plan.md`.
*   **Phases**:
    1.  **Discovery**: Map current implementation vs. objective.
    2.  **Drafting**: Propose structural changes (Traits, Types, Layers).
    3.  **Constraint Audit**: Verify the draft against the **KeyForge Law** (Hexagonal, Bit-perfect, etc.).
    4.  **Decomposition**: Break the plan into atomic GitHub Issues.

## 2. Constraints (Non-Negotiable)
*   **NO CODE CHANGES**: Planning mode is strictly read-only for source code.
*   **NO RALPH LOOPS**: The `ralph-loop` extension is STRICTLY FORBIDDEN in planning. Planning requires foresight and intent, not iterative trial-and-error.
*   **NO VIBE-TASKS**: Every GitHub Issue created must have a clear "Definition of Done" based on technical invariants.

## 3. The Planning State Manifest (`active_plan.md`)
Every planning session MUST maintain this header:
```markdown
# Active Plan: [Title]
*   **Issue ID**: #ID
*   **Status**: [Discovery | Drafting | Auditing | Ready]
*   **Objective**: [Statement]
*   **Constraint Check**: [Pass/Fail]
```

## 4. The 100x Handoff
Planning is complete ONLY when:
1.  All target files are identified.
2.  All new types/traits are defined in the plan.
3.  The plan is updated as a comment on the primary GitHub Issue.
4.  Sub-issues (if any) are created and linked.