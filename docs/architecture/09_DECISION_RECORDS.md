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
