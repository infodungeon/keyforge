# Operation Zero Entropy: The Unified Omni-Audit Plan

**Date:** 2026-01-24
**Version:** 3.0 (Omni-Integrated)
**Executor:** Gemini (Systemic Architect)

## I. Executive Summary
This document defines the "Zero-Point" audit protocol for the KeyForge ecosystem. It integrates all previously identified dimensions of technical debt with the specific architectural, operational, and design interrogations requested. We employ a multi-tool strategy (Narsil, Arbor, Ast-Grep, Cargo) to verify the system from its abstract strategy down to its binary footprint.

---

## II. The 10 Dimensions of Systemic Interrogation

### Dimension 1: Strategic & Domain Debt (The "Why")
*Evaluating the gap between business purpose and technical reality.*
*   **Technology Choice Suitability:** Questioning the stack (e.g., Is `sqlx`/`tauri` overhead justified for the current scale? Why `axum` for local asset serving?).
*   **Domain Divergence:** Identifying where code models (`KeyNode`, `Physics`) have drifted from the real-world keyboard domain.
*   **Zombie Features & Bloat:** Detecting features or projects (`apps/`, `libs/`) that are reachable but deliver zero marginal utility.
*   **Strategic Suitability:** Are we using frameworks where simpler libraries suffice? (Dependency weight vs. Usage count).

### Dimension 2: Architectural Physics & Structural Debt (The "Where")
*Interrogating the skeleton and boundaries.*
*   **Hexagonal Purity:** Detecting IO leakage (`std::fs`, `tokio`, `sqlx`) into core logic kernels. (Tool: **Narsil** graph).
*   **Layer Inversions:** Identifying if high-level logic depends on infra (Violation of ARCH-005).
*   **Coupling Gravity:** Using **Narsil** (PageRank) to find "God Objects" or modules that block refactoring through excessive dependencies.
*   **Module Topology:** Using **Arbor** to detect "Module Obesity" (files >500 LOC) and logical folder imbalances (e.g., "Drawer Directories" like `utils/`).
*   **Circular Debt:** Detecting indirect cycles between the 13 crates that impede compilation and modularity.

### Dimension 3: Design Pattern & Semantic Debt (The "How Well")
*Evaluating the quality of abstraction and reuse.*
*   **Anemic vs. Rich Models:** Do structs (`Layout`, `KeyNode`) enforce invariants at construction, or are they passive "bags of data"?
*   **Refactoring for Reuse:** Identifying componentization opportunities (e.g., moving shared logic from `apps/` to `libs/`).
*   **Extensibility & Maintainability:** Can new scoring algorithms or keyboard types be added without modifying the core `physics` engine? (Open/Closed Principle).
*   **Trivial Implementations:** Identifying manual boilerplate (`impl From`, `Clone`) that should be handled by structural or procedural macros (Violation of ARCH-006).
*   **Interface Debt:** Identifying where `pub` fields leak internal state instead of using clean accessor interfaces.

### Dimension 4: Type Safety & Coding Standards (The "Law")
*Enforcing idiomatic excellence and the "KeyForge Law".*
*   **Panic Pathways:** Using **Narsil** to trace every `unwrap()`, `expect()`, and `panic!()` to its public entry point. (Violation of TYPE-003).
*   **Primitive Obsession:** Hunting signatures like `fn(u16, u16) -> f32` where `KeyIndex`, `FingerIndex`, and `Score` are semantically required.
*   **Async Contagion:** Detecting "Color Pollution" where `async` is forced into CPU-bound paths.
*   **Safety Audit:** Interrogating `unsafe` blocks for documentation and invariant justification.

### Dimension 5: Verification & Determinism (The "Trust")
*Evaluating the confidence of system changes.*
*   **Assertion Quality & Mock Drift:** Are we testing reality or just verifying that our mocks work?
*   **Fixture Fragility:** Reliance on static/hardcoded JSON blobs vs. **Proptest** generators.
*   **Bit-Perfect Determinism:** Scanning for `f32`/`f64` usage in scoring/consensus paths where integer/fixed-point arithmetic is mandated.

### Dimension 6: Operational, Production & Infrastructure (The "Run")
*Evaluating the system's "deployment health".*
*   **Observability Blindspots:** Checking for structured logging (`tracing`) and unique request IDs in `keyforge-hive`.
*   **Config Hardcoding:** Hunting for "Magic Numbers" and hardcoded paths/secrets instead of centralized `config.toml` or ENV usage.
*   **Deployment Readiness:** Auditing `Dockerfile` and `docker-compose` for efficiency, security, and reproducibility.
*   **Error Mapping:** Verifying that production errors are mapped to structured types rather than generic strings (Violation of ARCH-007).

### Dimension 7: Cognitive & Documentation Debt (The "Knowledge")
*Bridging the gap between code and intent.*
*   **ADR Parity:** Matching major features to existing ADRs (Architectural Decision Records). Identifying "Silent Decisions."
*   **API Documentation:** Checking for `#[warn(missing_docs)]` compliance and the presence of examples.
*   **Knowledge Gaps:** Identifying complex modules with zero internal design documentation.

### Dimension 8: Supply Chain & Dependency Debt (The "Foundation")
*Interrogating the ground the system stands on.*
*   **Framework Tax Audit:** Evaluating if dependencies like `sqlx` or `tauri` are pull-ins that exceed the problem's requirements.
*   **Zombie Dependencies:** Using `cargo udeps` to find dependencies that are in `Cargo.toml` but never imported.
*   **Binary Bloat:** Using `cargo bloat` to find oversized dependencies (e.g., `aws-sdk` usage for trivial tasks).

### Dimension 9: Experience & UI Debt (The "Surface")
*Evaluating friction for the end-user.*
*   **UI Logic Leakage:** Verifying that `keyforge-ui` is a pure presenter and does not contain business/physics logic (Violation of ARCH-001).
*   **Consistency Audit:** Checking for divergent patterns between UI, TUI, and CLI interfaces.
*   **Latency Perception:** Identifying blocking IO operations on the UI thread.

### Dimension 10: Completeness & Maintenance (The "Attic")
*Cleaning out the systemic noise.*
*   **TODO Archeology:** Finding and dating all `TODO`, `FIXME`, and `HACK` comments.
*   **Unimplemented Logic:** Scanning for `todo!()` and `unimplemented!()` in reachable production paths.
*   **Dead Code Detection:** Identifying functions, structs, or constants never used in the **Narsil** graph.
*   **Churn Hotspots:** Identifying files with high git flux and high complexity for potential refactoring.

---

## III. Execution Matrix & Tooling

| Phase | Methodology | Primary Tools |
| :--- | :--- | :--- |
| **1. The MRI** | **Structural/Topological Audit**. Map couplings, layers, cycles, and "God Objects". | **Narsil** (Graph), **Arbor** (Topology) |
| **2. The Interrogation** | **Semantic/Standard Audit**. Panic tracing, Primitive checking, Design pattern review. | **Narsil** (Paths), **Ast-Grep** (Patterns) |
| **3. The Weigh-In** | **Tech/Dependency Audit**. Framework tax, Binary bloat, WASM compatibility. | **Cargo Tree/Udeps/Bloat** |
| **4. The Reality Check** | **Ops/Doc Audit**. Config scanning, ADR parity, Observability check. | **Grep**, **Manual Review**, **Narsil** |
| **5. Synthesis** | **Consolidation**. Generate prioritized `TECHNICAL_DEBT.md` and GitHub Issues. | **Markdown**, **GitHub CLI** |
