# KeyForge Project Context & Configuration

## 1. Project Identity
**KeyForge** is a high-performance, data-driven keyboard layout generation and analysis system.
**Core Philosophy:** Constraint-Based Engineering. We rely on type systems and compiler constraints to prevent errors, rather than hoping for logic to be correct.

## 2. Architecture & Design (Hexagonal)
Adhere strictly to the **Ports & Adapters** architecture:

*   **Tier 1: Core (Pure Logic)** - `keyforge-physics`, `keyforge-evolution`.
    *   **Constraints:** NO `std::fs`, `tokio`, or `sqlx`. Pure math/logic only.
    *   **Math:** Use **Fixed-Point Arithmetic** (`Score` type, `i64`) scaled by `1,000,000.0`. Never use `f32` for accumulation.
    *   **Types:** Strict usage of newtypes (`KeyIndex(usize)`, `FingerIndex(u8)`).
*   **Tier 2: Contracts (Ports)** - `keyforge-protocol` (DTOs), `keyforge-core` (traits).
    *   **Pattern:** Centralized Error Registry (`ForgeError`).
*   **Tier 3: Adapters (Infrastructure)** - `keyforge-infra`.
    *   **Pattern:** The Humble Object (keep logic out of IO code).
*   **Tier 4: Drivers** - `keyforge-hive` (Server), `keyforge-agent` (Worker), `keyforge-cli`.

## 3. Cognitive Patterns (Code Style)
*   **Typestate Pattern:** Encode state in types (`PendingJob` -> `RunningJob`).
*   **Parameter Object:** Group cohesive arguments (`ScoringContext { layout, corpus }`).
*   **Command Pattern:** Decouple Intent (`HiveCommand`) from Execution (`handle_command`).

## 4. Verification Strategy (The Guardrails)
*   **Rigorous Unit Testing:** All logic, edge cases, and math must be verified in `src/`. Target 95%+ branch coverage.
*   **Zero Duplication Policy:** Unit logic MUST NOT be re-verified in integration tests. Integration tests focus solely on "wiring" and cross-crate contracts.
*   **Crate Affinity:** Tests must reside in the crate that owns the integration point.
*   **The Oracle Pattern:** The optimized `ScoringEngine` MUST match the naive `DeterministicScorer` bit-for-bit.
*   **Mutation Testing:** `keyforge-physics` must pass `cargo mutants` (zero survived mutants).
*   **The "Cleanroom":** Debug complex logic in isolated reproduction scripts (`ops/repros/*.rs`), never inside the full app.

## 5. Worker Orchestration (Flash Protocol)
To ensure successful delivery by Gemini Flash, high-level plans must be decomposed into an **Atomic TaskList**.

**Execution Mandate for Flash:**
1.  **Atomicity:** Tasks must be single-purpose (e.g., "Add field to struct," "Implement one trait method").
2.  **Explicit Context Recall:** Before each task, state the Tier and Invariants.
3.  **Reason-First:** Explain the implementation logic in natural language *before* calling the tool.
4.  **Verification:** Every task must end with a specific verification command.

**Tactical Guardrails:**
*   **Fresh Eyes:** Always `read_file` immediately before editing to ensure line-perfect matching.
*   **Lift & Shift:** When moving code, copy it *verbatim* first (fixing only imports). Verify compilation. *Then* refactor. Never mix moving and fixing.
*   **Micro-Verification:** Run `cargo check -p <crate>` immediately after any file modification. Do not wait for the final checkpoint.

## 6. Operational Doctrine (The "SAGA" Protocol)
1.  **Isolate:** Create a reproduction case (`just repro <name>`).
2.  **Verify Failure:** Ensure the repro fails as expected.
3.  **Implement:** Write the minimal code to fix it (via Atomic TaskList).
4.  **Verify Success:** Run the repro/unit tests.
5.  **Reflect:** If a logic error occurred, explain *why* before fixing.

## 7. Automation & Tooling
*   **Task Runner:** Use `just` for all high-level operations.
    *   `just repro <name>`: Create a reproduction script template.
    *   `just context <file>`: Minify code for context window efficiency.
*   **Testing:**
    *   Unit: `cargo test -p <crate>`
    *   Integration: `cargo test --test <integration_test>`
    *   **Do not** use `cargo run` directly; use `just` commands to ensure environment variables are set.

## 8. Key Documentation References
*   `docs/architecture/00_MANIFESTO.md` - Engineering Principles.
*   `docs/architecture/11_SCORING_LOGIC.md` - Physics Math & Scoring Layers.
*   `docs/architecture/16_TESTING_STANDARDS.md` - Verification Protocols.
*   `docs/planning/refactor.md` - Active Refactor Roadmap.
