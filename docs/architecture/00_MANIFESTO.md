# KeyForge Engineering Manifesto (v6.1)

**Goal:** Engineering Truth, Deterministic State, & Operational Efficiency.
**Enemy:** Context Saturation, Implicit Logic, "Happy Path" Bias.
**Doctrine:** LLM-Integrated Development (LID).

## 1. Core Philosophy: Constraint-Based Engineering

We move from **Asking** (hoping the LLM is smart) to **Constraining** (making it impossible for the LLM to be wrong).

### Criticality Tiering

1. **Tier 1 (The Nucleus):** `keyforge-physics`, `keyforge-evolution`.
   * **Risk:** Mathematical hallucination.
   * **Requirement:** 95%+ Branch Coverage + Property Testing.
   * **Requirement:** Bit-perfect parity between Exact and Oracle implementations.
   * **Requirement:** 100% Checked Arithmetic in scoring kernels (No silent overflows).

2. **Tier 2 (The Contract):** `keyforge-protocol`, `keyforge-model`.
   * **Risk:** Data corruption.
   * **Requirement:** 100% Validation Coverage.

3. **Tier 3 (The Shell):** `keyforge-infra`, `keyforge-hive`.
   * **Risk:** IO/State failure.
   * **Requirement:** Error Path Verification.

## 2. Architectural Doctrine (Cognitive Ergonomics)

### A. The Semantic Firewall (Type-Driven Design)

1. **The Typestate Pattern (Compiler State Machine)**
   * **Problem:** Temporal Coupling (LLM accesses data before it is ready).
   * **Fix:** Encode state in types (`PendingJob` -> `RunningJob` -> `CompletedJob`).
   * **Rationale:** The compiler prevents accessing `score` on a `PendingJob`.

2. **The Parameter Object (Context Structs)**
   * **Problem:** Argument Swapping (LLM confuses `usize` arguments).
   * **Fix:** Group cohesive arguments into a Context Struct (`ScoringContext { layout, corpus }`).
   * **Rationale:** Fields are named; positional arguments are eliminated.

3. **The Centralized Error Registry**
   * **Problem:** Lazy error handling (`anyhow!`, `unwrap()`).
   * **Fix:** A strict, enumerated catalog (`ForgeError`) in `keyforge-protocol`.
   * **Rationale:** Forces the LLM to categorize failure modes (Physics vs Infra).

### B. Structural Patterns (Isolation)

1. **The Command Pattern (Reified Actions)**
   * **Problem:** Logic coupled to HTTP Handlers is hard to test.
   * **Fix:** Decouple Intent (`HiveCommand`) from Execution (`handle_command`).
   * **Rationale:** Enables testing business logic without spinning up a server.

2. **The Humble Object**
   * **Rule:** Extract logic from IO boundaries into pure structs.

3. **Vertical Slices**
   * **Rule:** Group code by Feature (`src/features/register_job`), not Layer.

4. **Railway Oriented Programming (ROP)**
   * **Rule:** Linear `Result` chains. No nested `if/else`.

## 3. Verification Strategy (The Guardrails)

### A. The Oracle Pattern (Multi-Tiered)

* **Exact Engine:** Must match `DeterministicScorer` bit-for-bit.
* **Search Engine:** May drift within documented bounds.
* **Protocol:** Tests must select the appropriate comparator based on the engine under test.

### B. Coverage-Driven Verification (CDV)

* **Goal:** Identify "Logic Dark Matter" (unexecuted branches).
* **Rule:** High coverage is a map of "Unknown Territory".

### C. Advanced Testing Protocols

1. **Mutation Testing (`cargo-mutants`):** Zero "Survived Mutants" allowed in Physics.
2. **Metamorphic Testing:** Verify properties (`dist(a,b) == dist(b,a)`).
3. **Golden Data:** Use hardcoded input/output pairs (`tests/fixtures/`) as the Anchor of Truth.
4. **Formal Verification Lite (`kani`):** Prove invariants for core math.
5. **Snapshot Testing (`insta`):** Lock regression for complex DTOs/Parsers.

### D. The Ghost Code Strategy

* **Rule:** For complex algorithms, implement a simplified "Ghost Model" first. Prove it, then optimize.

## 4. Operational Protocols (The Workflow)

### A. The SAGA Loop

1. **Isolation:** Debug in `repro.rs`.
2. **Tests First:** Test -> Fail -> Implement -> Pass.
3. **Signature Freeze:** No changing `pub` signatures during debugging.
4. **3-Turn Stop:** If fix fails twice: STOP, REVERT, ISOLATE.

### B. Context Management

1. **The Cleanroom:** Never debug inside the full app context.
2. **Context Compression:** Use "Header Files" (Signatures only) for dependencies.
3. **The Soft Reset:** Clear context window if reasoning degrades.

### C. The Reflexion Loop

* **Protocol:** When a test fails:
    1. **Input:** Failing Test + Code.
    2. **Task:** "Explain the logic error in natural language."
    3. **Action:** Generate fix based on explanation.

### D. Chain-of-Verification (CoVe)

* **Protocol:** Draft -> Verify (Manual Calc) -> Finalize.

## 5. Operational Reality (Day-2 Operations)

### A. Database Safety

* **Rule:** **Zero-Touch Migrations.**
* **Protocol:** Never write SQL migrations directly. Output schema, verify diff. Migrations must be Additive Only.

### B. Human Protocols

1. **The Comprehension Test:** Human must explain logic without reading LLM summary.
2. **The Two-Man Audit:** Coder != Auditor. Use fresh context for security audit.
3. **No Magic:** Reject macros/complex generics unless explicitly requested.

### C. Tiered Verification (CI Economics)

1. **Tier 1: The Fast Loop (Local):** `cargo check`, `cargo test` (Unit). Time: < 30s.
2. **Tier 2: The Logic Loop (PR):** `cargo test --doc`, `cargo mutants` (Diff). Time: < 10m.
3. **Tier 3: The Deep Freeze (Nightly):** `kani`, Full `cargo mutants`, E2E. Time: Hours.
