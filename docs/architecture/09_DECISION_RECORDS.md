# Architecture Decision Records (ADR)

**Version:** 4.3
**Context:** Log of significant architectural decisions.

## Index

* [ADR-001: Hexagonal Architecture](#adr-001-hexagonal-architecture)
* [ADR-002: No Async in Physics](#adr-002-no-async-in-physics)
* [ADR-003: Shared Secret Auth](#adr-003-shared-secret-auth)
* [ADR-004: Postcard for Deterministic Hashing](#adr-004-postcard-for-deterministic-hashing)
* [ADR-005: Feature Gating TypeScript Bindings](#adr-005-feature-gating-typescript-bindings)
* [ADR-006: Universal Domain Validation](#adr-006-universal-domain-validation)
* [ADR-007: Lowercase Alpha Normalization](#adr-007-lowercase-alpha-normalization)
* [ADR-008: Synthetic Corpus Injection](#adr-008-synthetic-corpus-injection)
* [ADR-009: Optimal Choice for Duplicate Keys](#adr-009-optimal-choice-for-duplicate-keys)

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
* **Context:** Domain entities were being deserialized at various boundaries (API, Disk, WASM) without consistent validation.
* **Decision:** Implement the `Validator` trait for **all** Domain Entities in `keyforge-model`. Enforce `.validate()` calls immediately after deserialization at **all** system boundaries.
* **Consequences:**
  * (+) Guarantees invalid data is caught at the IO boundary before entering the domain.
  * (+) Centralizes validation logic in the Model layer.
  * (-) Requires boilerplate `Validator` implementations for simple structs.

## ADR-007: Lowercase Alpha Normalization

* **Status:** Accepted
* **Date:** 2025-12-31
* **Context:** Keyboards send keycodes (e.g., `KC_A`), but text corpora usually contain lowercase characters ('a').
* **Decision:** Normalize all alphabetic keycodes to **lowercase** internally within the Domain Model. Uppercase is treated purely as a **Presentation Layer** concern.
* **Consequences:**
  * (+) Simplifies scoring logic.
  * (+) Consistent with standard text corpora.
  * (-) UI must explicitly uppercase keys for display if desired.

## ADR-008: Synthetic Corpus Injection

* **Status:** Accepted
* **Date:** 2026-01-02
* **Context:** Standard text corpora lack explicit `Enter` and `Backspace` events, leading to suboptimal placement of these keys.
* **Decision:** Inject synthetic frequency data for `Enter` and `Backspace` into `_std` corpora at load time based on error rate and punctuation models.
* **Consequences:**
  * (+) Layouts optimize for realistic typing flows, including errors and formatting.
  * (+) `Backspace` and `Enter` are pulled closer to the home row.
  * (-) Synthetic distribution is heuristic.

## ADR-009: Optimal Choice for Duplicate Keys

* **Status:** Accepted
* **Date:** 2026-01-06
* **Context:** Keyboards often feature duplicate keys (e.g., split spacebars or bilateral modifiers). Previous logic used a "last one wins" or "distributed load" approach, which failed to model an "optimal typist" who chooses the best physical key for a given context.
* **Decision:** Implement "Optimal Choice" logic. For every monogram, bigram, and trigram, the engine dynamically searches all physical instances of the involved characters to find the specific key (or combination) that yields the absolute minimum cost.
* **Consequences:**
  * (+) More realistic modeling of advanced and split layouts.
  * (+) Provides the architectural foundation for future stateful keys (e.g., Repeat key).
  * (+) Simplifies physics by removing `SpaceHandPreference` (now handled by the typist model).
  * (-) Increased computational complexity in the fast-path `score_layout` due to dynamic searching.
  