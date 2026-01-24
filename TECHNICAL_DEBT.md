# KeyForge Systemic Friction Map

## High-Heat Areas (Immediate Priority)
- **[SYMPTOM] Hive Data Mapping**: Manual conversion between SQLX/JSON and Model. Results in 27+ fragile errors.
  - *Pivot*: Implement `mapping.rs` projection trait in `keyforge-model`.
- **[SYMPTOM] Test Grace Proliferation**: Manual repetition of `#[allow]` in 89+ files.
  - *Pivot*: Harden `kf_test` macro to be the sole source of structural grace.

## Feature Gaps
- **[SYMPTOM] CLI Benchmark Incompleteness**: `apps/keyforge-cli/src/cmd/benchmark.rs` performs only raw throughput testing (KOPS). The detailed layout analysis metrics (SFB, Scissors, etc.) defined in the orphaned `BenchmarkEntry` struct (`apps/keyforge-cli/src/reports/benchmarks.rs`) are not integrated into the reporting pipeline. The "Reality Check" comparison table is effectively dead code.
  - *Pivot*: Re-integrate `BenchmarkEntry` into the `benchmark` command's output generation or creating a dedicated reporting stage that utilizes `keyforge-model::metrics`.

## Functionality Stubs & Dead Code
- **[STUB] Job Status Aggregation**: `apps/keyforge-hive/src/features/get_job_status.rs` has `total_compute_sec` hardcoded to 0. Requires DB aggregation logic.
- **[STUB] Agent CPU Detection**: `apps/keyforge-agent/src/agent/network/manager.rs` uses a placeholder string for CPU model. Needs real hardware detection integration.
- **[BROKEN] CLI Search Reporting**: `apps/keyforge-cli/src/cmd/search.rs` has reporting logic commented out because `keyforge_compute::analyze_layout` is missing.
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

## Protocol for Debt
1. **Never delete findings.**
2. **Never condense granular points into high-level summaries.**
3. **If a build fails, record the "Structural Reason" why it was possible here.**
