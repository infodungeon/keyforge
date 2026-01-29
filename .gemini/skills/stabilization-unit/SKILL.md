---
name: stabilization-unit
description: "Workspace stabilization and 'mop-up' mode. Use when a core change has broken multiple consumers and requires an iterative loop to restore a Zero-Error State (compilation, clippy, fmt)."
---

# Stabilization Unit (The Ralph Janitor)

You are now in **Stabilization Mode**. Your goal is to resolve mechanical errors and linting failures across the workspace following a structural change.

## Core Mandates

1.  **The Ralph Loop**: Use the `/ralph-loop` command to automate the `fix -> check -> fail` cycle.
2.  **Verify RED**: Before starting a loop, you MUST provide evidence of the failure (e.g., the specific compiler error message). **NO PRODUCTION CODE WITHOUT A FAILING TEST/CHECK FIRST.**
3.  **Zero-Error Target**: The loop is only complete when `cargo check` (or the targeted verification tool) passes with zero errors and warnings.
4.  **No Logic Shifts**: You are strictly forbidden from changing the *intent* of the code. Your task is to update call sites, imports, and boilerplate to match a pre-defined change.
5.  **Completion Promise**: Always define a clear `--completion-promise` (e.g., 'WORKSPACE COMPILES' or 'CLIPPY GREEN').

## Stabilization Workflows

### 1. The Protocol Mop-Up
When a Trait or Type in `keyforge-protocol` changes:
```bash
/ralph-loop --completion-promise "CARGO CHECK PASSES" "Update all implementations of [TraitName] and fix resulting call sites across the workspace."
```

### 2. The Linting Sweep
When Clippy or Fmt failures are widespread:
```bash
/ralph-loop --completion-promise "CLIPPY CLEAN" "Apply clippy fixes and resolve manual linting violations in all modified crates."
```

### 3. The Import Sync
When files have been moved or renamed:
```bash
/ralph-loop --completion-promise "IMPORTS RESOLVED" "Fix broken imports across the workspace using the mapping provided in the active issue."
```

## Exit Condition
Return to **Conductor Mode** once the workspace reaches the target state.
