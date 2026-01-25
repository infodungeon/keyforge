# KeyForge Systemic Friction Map

## High-Heat Areas (Immediate Priority)
- [x] **[SYMPTOM] Hive Data Mapping**: Manual conversion between SQLX/JSON and Model. Results in 27+ fragile errors.
  - *Pivot*: Implement `mapping.rs` projection trait in `keyforge-model`. (Resolved via Projection Bundle pattern in Issue #39)
- **[SYMPTOM] Test Grace Proliferation**: Manual repetition of `#[allow]` in 89+ files.
  - *Pivot*: Harden `kf_test` macro to be the sole source of structural grace.

## Feature Gaps
- [x] **[SYMPTOM] CLI Benchmark Incompleteness**: `apps/keyforge-cli/src/cmd/benchmark.rs` performs only raw throughput testing (KOPS). Integrated metric reporting and reality check (Issue #38).
- [x] **[BROKEN] CLI Search Reporting**: `apps/keyforge-cli/src/cmd/search.rs` has reporting logic restored and unified with benchmark reporting (Issue #38).
- **[DEAD] Reporting Tables**: `apps/keyforge-cli/src/reports/tables.rs::scoring` is currently unused due to the broken search reporting.
- **[DEAD] Duplicate Loader**: `libs/keyforge-adapter/src/loader.rs` is an untracked/dead file that duplicates `libs/keyforge-model/src/loader.rs`.
- **[DISABLED] Kani Verification**: `libs/keyforge-model/src/verification.rs` is an untracked/orphaned file. Formal verification proofs are not currently running.

## Architectural Pressure Points
- **Physics Kernels**: Repetitive scoring loops.
  - *Pivot*: Collapse into generic `ScoringEngine` trait.

## Systemic Root Cause Analysis
### 1. Ambiguous Hexagonal Boundaries (The "Loader" Conflict)
- **Symptoms**: Duplicate `loader.rs` in `adapter` vs `model`; Broken CLI reporting due to API drift.
- **Diagnosis**: The boundary between "Adapter" (IO) and "Model" (Pure Data) is blurred. Asset loading logic (IO) is leaking into the Model crate, while Reporting logic (Application Service) is coupled to the CLI.
- **Remediation**: 
    - Move *all* IO/Loading logic to `libs/keyforge-adapter`.
    - Ensure `libs/keyforge-model` is pure data types only.
    - Establish `libs/keyforge-compute` as the stable Application Facade.

### 2. Missing Anti-Corruption Layer (ACL)
- **Symptoms**: Fragile Hive Data Mapping (SQLx <-> Model).
- **Diagnosis**: Database schema details are leaking directly into domain logic, necessitating manual, error-prone mapping.
- **Remediation**: Implement a strict `Projection` trait (ACL) to automate and strictly type the conversion between DTOs and Entities.

### 3. Incomplete System Abstraction
- **Symptoms**: `#[allow]` proliferation; Repetitive scoring loops.
- **Diagnosis**: Common patterns (testing, iteration) are solved manually via copy-paste rather than structurally via Macros or Traits.
- **Remediation**: Enforce "Structural Oracle" pattern: use `kf_test` macro for all test configurations and `ScoringEngine` trait for all physics loops.

## Audit Zero Entropy (2026-01-24) - Grand Unified Report

### 1. Architectural & Structural Findings
- **[ARCH-CRITICAL] Module Obesity**: `libs/keyforge-model/src/config/weights.rs` (783 lines) and `libs/keyforge-model/src/types.rs` (744 lines) exceed the 500 LOC threshold.
- **[ARCH-005] Hidden IO**: CORS origins hardcoded in `apps/keyforge-hive/src/lib.rs`.
- **[COUPLING] God Functions**: `new`, `default`, and `validate` (291 connections) are structural bottlenecks.
- [x] **[DEAD] Orphaned Exports**: 107 unused public exports detected in the global graph. (Resolved by downgrading concrete engines and compilation machinery to pub(crate) in Issue #36)

### 2. Semantic & Safety Findings
- **[SAFETY-RISK] Undocumented Unsafe**: 35 unsafe blocks found; many lack `// Safety:` justification (e.g. `apps/keyforge-agent/src/hw_detect.rs`).
- **[TYPE-003] Panic Reachability**: `calculate_swap_delta` traces to 12 internal `unwrap()` calls. 
- **[PRIMITIVE] Float Leakage**: 58 functions in `keyforge-compute` and `weights.rs` return raw `f32/f64` instead of `Score` or `Weight` newtypes.
- **[ANEMIC] passive Data**: 107 passive structs in `keyforge-protocol` leak internal state via `pub` fields.

### 3. Operational & Observability Findings
- **[OBSERVABILITY] Instrument Void**: `#[instrument]` is missing from 98% of public entry points in `keyforge-hive` and `keyforge-physics`.
- [x] **[CONFIG] Magic Literals**: 42 instances of hardcoded numeric literals (magic numbers) > 1000 found in \`libs/keyforge-physics/src/engines\`. Elevated to \`EngineConfig\` schema in Issue #35.
- [x] **[OPS] Docker Bloat**: \`ops/Dockerfile.*\` lacks multi-stage builds and explicit user non-root escalation. (Resolved via Issue #37)

### 4. Supply Chain Findings
- **[DEP] Framework Tax**: `sqlx` and `tokio` are over-prescribed in `keyforge-infra` for tasks that could be handled by `rusqlite` or `std::sync`.
- **[DEP] Supply Chain Churn**: `keyforge-ui/src-tauri` depends on 400+ transitive crates for a simple desktop wrapper.

### 5. Maintenance Findings
- **[CHURN] Hotspots**: `libs/keyforge-physics/src/kernel/compute/delta.rs` is the #1 churn hotspot (623 lines, 45+ commits).
- **[STUB] TODO Archeology**: `TODO: Aggregate from DB` found in `get_job_status.rs` remains unimplemented.

## Protocol for Debt
1. **Never delete findings.**
2. **Never condense granular points into high-level summaries.**
3. **If a build fails, record the "Structural Reason" why it was possible here.**