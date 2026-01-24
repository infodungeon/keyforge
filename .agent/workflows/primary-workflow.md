# 100x Execution Protocol: The Success-Trap

## 1. The Prototypical Slice (Pre-Batch Verification)
- **Mandate**: Never perform a batch operation (multiple files) until the "Master Pivot" has been proven in a single file.
- **Verification**: Run `cargo clippy` and `cargo test` on the slice. If it doesn't simplify the code or resolve the root cause, abort the batch.

## 2. Friction Triggers (When to Think)
- **Spatial Friction**: Logic change requires > 3 files. Action: Identify the centralized trait/macro pivot.
- **Repetitive Friction**: Manual data mapping or boilerplate detected. Action: Implement a generative/projection solution.
- **Tool Friction**: Script or regex fails once. Action: Immediately shift to AST-aware tools (ast-grep, syn macros).

## 3. The Stable Ground State (Turn Anatomy)
- **Simulation**: Map ripple effects using `codebase_investigator` before acting.
- **Execution**: Apply surgical changes using high-precision tools.
- **Hard Gate**: Run `just 100x` (Audit -> Gate -> Clippy Deny).
- **Turn Bound**: No turn is complete until the workspace is Build-Clean, Lint-Clean, and Abstraction-Simplified.

## 4. Evolutionary Learning
- **Gate Hardening**: If a build fails, update the `Justfile` or `ast-grep` rules to block that *class* of error before fixing the instance.
- **Fragility Mapping**: Record the structural reason for any failure in `TECHNICAL_DEBT.md`.