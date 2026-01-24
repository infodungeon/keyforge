# KeyForge Constraint Registry (The "Law")

This document defines the strictly enforced architectural, type-system, and operational constraints of the KeyForge codebase. Violating these rules is considered a build failure.

**Status Legend:**
- 🔴 **ACTIVE**: This constraint is currently being enforced.
- 🟢 **RESOLVED**: The underlying issue has been structurally fixed.

## 1. Architectural Constraints (The "No-Go" Zones)

| ID | Status | Component | Constraint | Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **ARCH-001** | 🔴 | `keyforge-ui` | **No Data Transformation in Components** | React components must receive ready-to-render data. All parsing/adapting (e.g., `UiCategoryMap` -> `CategoryData`) must happen in `src/api/client.ts` or the Backend. **DO NOT write "adapters" in `useEffect`.** |
| **ARCH-002** | 🔴 | `keyforge-hive` | **No "Fat Handlers"** | Axum handlers (in `features/`) must only deserialize input and call a Service. Business logic in handlers is forbidden. |
| **ARCH-003** | 🔴 | `keyforge-physics` | **No Floating Point Accumulators** | Scoring accumulation must use integer arithmetic (fixed point) to guarantee determinism across CPU architectures. |
| **ARCH-004** | 🔴 | `Global` | **No Raw SQL** | All database interactions must use `sqlx::query!` or `sqlx::query_as!` macros for compile-time verification. String concatenation for SQL is banned. |
| **ARCH-005** | 🔴 | `keyforge-core` | **No Direct IO in Kernels** | Physics kernels must be pure functions. They cannot read files, access the network, or print to stdout (except logging). |

## 2. Type System Constraints (The "Safety" Net)

| ID | Status | Component | Constraint | Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **TYPE-001** | 🔴 | `keyforge-ui` | **No `any` in API Clients** | API response types must be strictly defined in `src/types/generated`. If the backend response doesn't match, fix the Type, not the code. |
| **TYPE-002** | 🔴 | `keyforge-model` | **No Primitive Obsession for Keys** | Use `KeyCode`, `KeyIndex`, `RowIndex` newtypes. Never pass raw `u8` or `usize` for key identifiers. |
| **TYPE-003** | 🔴 | `Global` | **No `unwrap()` in Production Code** | `unwrap()` is allowed ONLY in tests (`#[cfg(test)]`) or `ops/repros`. Production code must handle errors. |
| **TYPE-004** | 🔴 | `keyforge-ui` | **Strict Config Types** | `DEFAULT_APP_CONFIG` must satisfy the generated `Config` interface. Do not cast objects to `any` to bypass missing properties. |

## 3. Operational Constraints (The "Workflow")

| ID | Status | Component | Constraint | Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **OPS-001** | 🔴 | `CI` | **Artifact Hygiene** | CI jobs must clean up large artifacts (Android SDK, Dotnet) to prevent "No space left on device". |
| **OPS-002** | 🔴 | `Database` | **Migration Integrity** | Never modify an existing migration file. Always create a new versioned migration. |
| **OPS-003** | 🔴 | `Assets` | **Source of Truth** | `keyforge-assets` serves files from `data/`. Do not hardcode asset data (like key categories) in frontend code. |
