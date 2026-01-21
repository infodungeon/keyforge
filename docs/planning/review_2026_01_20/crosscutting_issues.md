# KeyForge Cross-Cutting Issues (Architecture Audit)

**Date:** 2026-01-20

## 1. String-Based Semantic Coupling
- **Deficiency**: Logic driven by string matching (`contains("_std")`, `starts_with`).
- **Impact**: High fragility to asset renaming.
- **Affected**: `infra`, `physics`, `testing`.

## 2. Allocation in Hot Paths
- **Deficiency**: Heap allocations (`Box`, `Vec::clone`) in inner loops.
- **Impact**: Performance degradation (GC pressure, allocator contention).
- **Affected**: `physics`, `evolution`, `agent`.

## 3. Heuristic Over-Reliance
- **Deficiency**: Hardcoded magic numbers (150ms, 0.5 balance, 0.2 similarity).
- **Impact**: Incorrect results for non-standard use cases (accessibility, ergonomic splits).
- **Affected**: `model`, `compute`, `adapter`.

## 4. Incomplete Error Registry
- **Deficiency**: Use of `anyhow` and string errors in libraries.
- **Impact**: API consumers cannot handle specific failure modes.
- **Affected**: `protocol`, `persistence`, `runner`.

## 5. Contract Precision Drift
- **Deficiency**: `f32` vs `f64` vs `i64` (fixed). TS binding to `number`.
- **Impact**: Determinism failure, UI data corruption.
- **Affected**: `protocol`, `model`, `ui`.
