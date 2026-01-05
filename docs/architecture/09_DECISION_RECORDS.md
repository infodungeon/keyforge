# Architecture Decision Records (ADR)

**Version:** 4.2
**Context:** Log of significant architectural decisions.

## Index

* [ADR-001: Hexagonal Architecture](#adr-001-hexagonal-architecture)
* [ADR-002: No Async in Physics](#adr-002-no-async-in-physics)
* [ADR-003: Shared Secret Auth](#adr-003-shared-secret-auth)
* [ADR-004: Postcard for Deterministic Hashing](#adr-004-postcard-for-deterministic-hashing)
* [ADR-005: Feature Gating TypeScript Bindings](#adr-005-feature-gating-typescript-bindings)
* [ADR-006: Universal Domain Validation](#adr-006-universal-domain-validation)
* [ADR-007: Lowercase Alpha Normalization](#adr-007-lowercase-alpha-normalization)

---

## ADR-001: Hexagonal Architecture

* **Status:** Accepted
* **Date:** 2024-01-01
* **Context:** We need to swap databases and frontends without rewriting core logic.
* **Decision:** Use Ports & Adapters. `keyforge-physics` must have ZERO dependencies on `keyforge-infra`.
* **Consequences:**
  * (+) Core logic is purely testable.
  * (-) Boilerplate for DTO conversion.

## ADR-002: No Async in Physics

* **Status:** Accepted
* **Date:** 2024-01-15
* **Context:** Scoring is CPU-bound. Async overhead slows down tight loops.
* **Decision:** `keyforge-physics` and `keyforge-evolution` will be synchronous.
* **Consequences:**
  * (+) Maximum CPU throughput.
  * (-) Workers must use `spawn_blocking` to prevent blocking the reactor.

## ADR-003: Shared Secret Auth

* **Status:** Accepted
* **Date:** 2024-02-01
* **Context:** We need a simple way to secure a private cluster. OAuth is overkill.
* **Decision:** Use a static Shared Secret (Bearer Token).
* **Consequences:**
  * (+) Simple to implement.
  * (-) Key rotation requires restarting all nodes.

## ADR-004: Postcard for Deterministic Hashing

* **Status:** Accepted
* **Date:** 2025-12-30
* **Context:** `bincode` is unmaintained. We need a stable, deterministic binary serialization format for generating `JobIdentifier` hashes.
* **Decision:** Replace `bincode` with `postcard`.
* **Consequences:**
  * (+) Maintained, `no_std` compatible, designed for embedded/deterministic use.
  * (-) Breaking change for existing Job IDs (hashes will change).

## ADR-005: Feature Gating TypeScript Bindings

* **Status:** Accepted
* **Date:** 2025-12-30
* **Context:** `ts-rs` emits warnings about `#[serde]` attributes it doesn't understand (like `deserialize_with`). These warnings pollute the build log.
* **Decision:** Gate `ts-rs` behind a `ts_bindings` feature flag.
* **Consequences:**
  * (+) Clean build logs for standard development (`cargo test`).
  * (+) Reduced dependency footprint for production builds.
  * (-) Developers must explicitly run `cargo test --features ts_bindings` to verify bindings.

## ADR-006: Universal Domain Validation

* **Status:** Accepted
* **Date:** 2025-12-31
* **Context:** Domain entities were being deserialized at various boundaries (API, Disk, WASM) without consistent validation. Some entities relied on inherent methods, others on traits, and some had no validation at all, leading to potential "shotgun parsing" vulnerabilities.
* **Decision:** Implement the `Validator` trait for **all** Domain Entities in `keyforge-model`. Enforce `.validate()` calls immediately after deserialization at **all** system boundaries (`keyforge-infra`, `keyforge-hive`, `keyforge-wasm`).
* **Consequences:**
  * (+) Guarantees invalid data is caught at the IO boundary before entering the domain.
  * (+) Centralizes validation logic in the Model layer (Single Source of Truth).
  * (+) Ensures consistency across different drivers (CLI vs Web vs Server).
  * (-) Requires boilerplate `Validator` implementations for simple structs.

## ADR-007: Lowercase Alpha Normalization

* **Status:** Accepted
* **Date:** 2025-12-31
* **Context:** Keyboards send keycodes (e.g., `KC_A`), but text corpora usually contain lowercase characters ('a'). Mixing case in the physics engine complicates frequency analysis and scoring.
* **Decision:** Normalize all alphabetic keycodes to **lowercase** internally within the Domain Model (`KeycodeRegistry`, `Layout`, `Corpus`). Uppercase is treated purely as a **Presentation Layer** concern (UI rendering).
* **Consequences:**
  * (+) Simplifies scoring logic (no need to check both 'A' and 'a').
  * (+) Consistent with standard text corpora.
  * (-) UI must explicitly uppercase keys for display if desired.
  * (-) Parsers must normalize input (e.g., "A" -> `KeyCode(97)`).

## ADR-008: Synthetic Corpus Injection

* **Status:** Accepted
* **Date:** 2026-01-02
* **Context:** Standard text corpora (e.g., books, articles) lack explicit `Enter` and `Backspace` events. Optimization engines trained solely on this data tend to place these critical keys in suboptimal positions (e.g., far corners), ignoring the reality of error correction and line breaking.
* **Decision:** Inject synthetic frequency data for `Enter` and `Backspace` into `_std` (Standard Prose) corpora at load time.
* **Backspace:** Calculated as `TotalChars * 0.03 (Error Rate) * 1.25 (Correction Factor)`. Distributed as `Char -> Bksp` transitions.
* **Enter:** Calculated as `SentenceCount / 3.0 (Sentences per Paragraph)`. Distributed as `Punctuation -> Enter` transitions.
* **Consequences:**
* (+) Layouts optimize for realistic typing flows, including errors and formatting.
* (+) `Backspace` and `Enter` are pulled closer to the home row/strong fingers.
* (-) Synthetic distribution is heuristic; may not perfectly model individual user behavior (e.g., "spamming" backspace vs. long-press).
