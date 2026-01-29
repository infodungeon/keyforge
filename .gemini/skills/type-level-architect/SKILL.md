---
name: Type-Level Architect
description: Enforces advanced TypeScript patterns for contract safety and domain-driven design.
version: 1.0.0
---

# Type-Level Architect

Enforces advanced TypeScript patterns for contract safety and domain-driven design.

## Instructions

### 1. Branded Types (Domain Integrity)
- Use branded types for all domain IDs (e.g., `CorpusId`, `LayoutId`) to prevent illegal mixing of primitive strings/numbers.
- Example: `type CorpusId = string & { readonly __brand: 'CorpusId' }`.

### 2. Contract-Safe DTOs
- Use `Pick<T, K>` and `Omit<T, K>` to derive DTOs from domain models.
- NEVER expose internal domain state or metadata in public API responses.
- Enforce `Readonly<T>` for all props and state objects to ensure immutability.

### 3. Exhaustive Handling
- Use the `never` type to ensure exhaustive pattern matching in `switch` statements or conditional types.
- Implement strictly typed Event/Action maps for reducers and event buses.

### 4. Utility Leverage
- Use `Required<T>` and `NonNullable<T>` for validated data paths.
- Leverage template literal types for string-pattern validation (e.g., color hex codes, version strings).

## Verification
- Run `tsc --noEmit` to verify type integrity.
- Ensure zero usage of `any` or `unsafe` casts in domain logic.