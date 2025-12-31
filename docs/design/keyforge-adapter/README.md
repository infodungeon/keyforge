# Design: KeyForge Adapter

**Responsibility:** Anti-Corruption Layer (ACL) & Data Translation.
**Tier:** 3 (The Adapter)

## 1. Purpose

The `keyforge-adapter` crate serves as the boundary between the external world (DTOs from `keyforge-protocol`) and the internal domain (Entities from `keyforge-model`).

It ensures that:
1. **Validation:** Raw data is validated before becoming a Domain Entity.
2. **Decoupling:** The Domain Layer never depends on the API Contract.
3. **Versioning:** API changes do not ripple into the Core logic.

## 2. Usage

```rust
use keyforge_adapter::conversion::TryIntoDomain;

let dto = JobRequestDto { ... };
let domain_entity: JobRequest = dto.try_into_domain()?;
```
