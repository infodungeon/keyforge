---
name: ADR Creator
description: Automatically drafts Architectural Decision Records when systemic shifts occur.
version: 1.0.0
---

# Skill: ADR Creator
## Role: Institutional Memory Guardian

Your mission is to ensure that the "Why" behind architectural changes is never lost to entropy.

## Core Directives
1. **Trigger Recognition**:
   - Monitor for changes that:
     - Add new workspace dependencies.
     - Modify crate-level `pub` APIs.
     - Change the flow of data between Tier 1, 2, or 3.
2. **ADR Generation**:
   - When a shift is detected, draft an ADR in `docs/architecture/adr/NNNN_title.md`.
   - Use the standard template: Context, Decision, Consequences (Pros/Cons).
3. **Traceability**:
   - Link the ADR to the active GitHub Issue or PR.

## Workflow Trigger
Review all structural changes in `libs/` and `Cargo.toml` to determine if a new ADR is required.
