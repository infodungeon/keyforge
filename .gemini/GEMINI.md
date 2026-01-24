# KeyForge Engineering Manifesto (v100.0) — THE SYSTEMIC ARCHITECT

**Core Directive:** Engineering Truth is achieved through **Systemic Invariants**, not checklists.

## 1. The Three Pillars of 100x Leverage

### I. Semantic Certainty
- **Pattern:** Use structural analysis (`ast-grep`) to enforce intent across the entire codebase.
- **Principle:** If a constraint is violated once, assume it is violated everywhere. Audit the *pattern*, not the file.

### II. The Execution State Machine (ESM)
- **Pattern**: Manage all tool calls via a state-transition model (`DISPATCHED` -> `MONITORED` -> `BACKGROUNDED` | `TERMINATED`).
- **Principle**: The CLI is for **Decision**. The Shell is for **Stateful Execution**.
- **Systemic Async**: Commands with high algorithmic complexity (e.g., O(2^n) proofs) or uncertain latency are backgrounded by default.
- **The Reconciliation Loop**: Every interaction begins with a **State Audit** of background tasks. Orphaned processes are a systemic failure.

### III. Correct-by-Construction
- **Pattern:** Encode business logic into the Type System (Typestates, Newtypes).
- **Principle:** A bug that can be represented in the Type System is an architectural failure.

## 2. Operational Heuristics

- **Context Optimization**: Use `minify_context.py` to keep the "Active Surface" minimal. Noise is the enemy of precision.
- **Predictive Refinement**: When a tool fails, don't just "fix it." Analyze the **Failure Mode** (Context, Logic, or Tooling) and update the strategy.
- **Zero-Trust Context**: Never assume a file's content or a library's behavior. Verify with `read_file` or `sg` before acting.

## 3. The 100x Bouncer (Conceptual)
1. **No Panics**: Total error propagation via `ForgeError`.
2. **No Info-Erasure**: Structured error mapping only.
3. **Deterministic Physics**: Bit-for-bit parity with the Oracle.
4. **Layer Purity**: Strict adherence to the Tiered Architecture.
