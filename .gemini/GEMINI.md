<<<<<<< HEAD
# KeyForge 100x Systemic Constitution (Final)

**Governing Strategy:** Hexagonal Purity & Parallel Intelligence
**Core Directive:** Engineering Truth is achieved through **Systemic Invariants** and **Orchestrated Specialization**.

---
=======
# KeyForge Systemic Governance

**Core Mandate:** All agents operating within the KeyForge project MUST read and strictly adhere to the architectural, performance, and testing invariants defined in the project constitution.
>>>>>>> master

## 1. The Systemic Law
- **Canonical Source:** `@keyforge_law.yaml`
- **Technical Standards:** `@AGENTS.md`
- **Active Roadmap:** `@active_plan.md`
- **Enforcement:** Every agent interaction must begin by verifying compliance with these rules. No action shall be taken that violates the ARCH, TYPE, PERF, or TEST laws.

<<<<<<< HEAD
### I. The Workflow Oracle (CRITICAL)
- **Rule:** Every agent interaction MUST begin by reading the local `.gemini/TASK_WORKFLOW.md`.
- **Constraint:** This is the canonical source of operational truth for your specific role.

### II. Protocol 0: Knowledge Acquisition (THE FOUNDATION)
All agents MUST read the entire documentation suite upon session initiation to achieve absolute contextual alignment:
- **Architecture:** Every file in `docs/architecture/`.
- **ADRs:** Every Architectural Decision Record in `docs/architecture/adr/`.
- **Design:** Every file in `docs/design/` (including all app and lib READMEs).
- **Environment Discovery:** `.gemini/settings.json`, `mcp_config.json`, `Justfile`, and `Cargo.toml`.

### III. The Intelligence Protocol (Swarm v2.0 - ACTIVE)
- **Status:** **ONLINE**. Use `swarm_submit` (Non-Blocking) for all tasks > 5s.
- **The Council:** Use `swarm_submit` with multiple targets (`sambanova_r1`, `groq_pro`) for deep reasoning.
- **CRITICAL:** Swarm models are prone to hallucination. All insights MUST be validated against source code facts.

### IV. Protocol 1: Global Communication
All agents must communicate with the **Conductor** for handovers and status updates.
- **Tool:** Use `send_prompt(target_instance: "Conductor", prompt: "...")`.
- **Readiness:** Signal readiness with `[AGENT_NAME]_IDLE`.
- **Handovers:** Provide deliverables (branch names, findings, or audit reports) directly to the Conductor.

### V. Protocol 2: The Intelligence Grid (MCP)
- **`narsil`:** Code Intelligence (Call graphs, symbols, data flow).
- **`arbor`:** Structural Oracle (Workspace geometry, crate dependencies).
- **`github`:** Project Management (Issues, PRs, CI status).
- **`swarm`:** Parallel Reasoning (Council consensus).

---

## 2. The KeyForge Law (ARCH-00x)

- **ARCH-001 (UI Purity):** Zero data transformation in React components.
- **ARCH-002 (Slim Handlers):** API handlers must be < 10 lines. Delegate to domain services.
- **ARCH-003 (Deterministic Physics):** Zero floating-point accumulators in kernels. Use `i64` fixed-point math exclusively.
- **ARCH-004 (Compile-Time SQL):** No raw SQL strings. Use `sqlx::query!` for static verification.
- **ARCH-005 (Hexagonal Purity):** No direct IO (filesystem, network, DB) in logic kernels. Use trait injection.
- **ARCH-006 (Structural Oracle):** Literal strings for system nouns (e.g., "model_ortho") are FORBIDDEN. Use macros/constants.
- **SEARCH-001:** DO NOT perform root searches on `./`. Use specific filters.

---

## 3. Technical Invariants

### Performance (PERF-00x)
- **PERF-001:** Any array/slice > 1,024 elements MUST be wrapped in `Arc<[T]>`.
- **PERF-002:** Zero `.clone()` calls in hot loops (physics/evolution).
- **PERF-003:** EngineContext must be immutable once compiled.

### Type-Safety & Reliability (TYPE-00x)
- **TYPE-001/3 (Panic-Free):** Zero use of `unwrap/expect/panic/todo` in production logic. Permitted ONLY in tests (`#[cfg(test)]`) and managed via `clippy`.
- **TYPE-002:** All fallible operations must return `Result` using `ForgeError` or crate-specific enums.
- **Newtypes:** Use `KeyIndex`, `Score`, etc., to prevent primitive obsession.

---

## 4. Operational Heuristics

- **Zero-Trust Context:** Run `read_file` on struct/trait definitions before usage.
- **Diagnostic Pivot:** After 2 failed remediation attempts, consult the Swarm Council.
- **Tooling Purity:** Always `read_file` before `write_file`.
- **Batching Mandate:** If `cargo check` returns N errors, fix all N before re-verifying.
- **ETS-100x (Ticket Standard):** Every change must be traceable to a self-contained GitHub Issue containing:
    - **Context:** Precise file paths and connectivity logs.
    - **Evidence:** Literal code snippets or trace data showing the violation.
    - **Deliverable:** A clear "Definition of Done".
- **CI-001:** Mandatory Continuous Improvement Audit at every track closure.
=======
## 2. Shared Discovery
- **Architecture:** Read `docs/architecture` for the definitive structural rules and invariants.
- **Design:** Read `docs/design` for established design patterns and component mappings.

## 3. Shared Knowledge
- **Hexagonal Purity:** Maintain rigid boundaries between Kernels, Protocols, and Adapters.
- **Deterministic Physics:** Integer-only math (`Score`) in all physics logic.
- **Compile-Time Safety:** Use macros for SQL and strict error propagation.
>>>>>>> master
