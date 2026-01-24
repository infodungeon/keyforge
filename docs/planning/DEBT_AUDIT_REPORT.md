# Workspace Technical Debt Audit Report (2026-01-23)

This report identifies exhaustive technical debt findings across the KeyForge workspace. This data serves as the basis for the next "Scrubbing" phase.

## 1. Safety & Integrity Debt (High Risk) 🔴

### 1.1 Unsafe Error Handling (`unwrap`/`expect`)
There are **~50 instances** of `.unwrap()` in production-path code (excluding tests).
- **Critical Hotspot**: `libs/keyforge-physics/src/analysis/mod.rs` uses `.unwrap()` for layout creation and scoring. A malformed layout in production will crash the entire process.
- **Critical Hotspot**: `libs/keyforge-physics/src/analysis/heuristics.rs` uses `.unwrap()` in the `Compiler::compile` path.
- **Risk**: Any malformed asset or user input will trigger a panic in the physics nucleus.

### 1.2 Information Erasure (Error Masking)
- **Pattern**: Widespread use of `.map_err(|e| e.to_string())`.
- **Primary Offenders**: `keyforge-infra/src/net/sync.rs`, `keyforge-model/src/utils/json.rs`.
- **Debt**: By the time an error reaches the UI, it is a generic string. We cannot distinguish between "Disk Full", "Network Timeout", or "Invalid Hash".

---

## 2. Architectural Debt (Layer Violations) 🟡

### 2.1 Dependency Fragmentation
- **Fragmentation**: `apps/keyforge-ui/src-tauri` and `apps/keyforge-cli` use hardcoded versions for `tokio`, `serde`, and `opentelemetry`.
- **Rogue Crate**: `libs/keyforge-wasm` uses `getrandom = "0.3"` while the rest of the workspace uses `0.2` (via transitive dependencies).
- **Tooling Debt**: `ops/repros` and `tests/system` have their own version tracking, leading to duplicate compilations of heavy crates like `tokio`.

### 2.2 Layer Purity (Domain Logic Side-Effects)
- **Finding**: No direct `std::fs` or `std::net` found in `physics` or `evolution` (SUCCESS).
- **Finding**: `keyforge-persistence` still has a dependency on `keyforge-compute` in its test-suite which could leak into production if not careful.

---

## 3. Performance Debt (Optimization Gaps) 🟢

### 3.1 Hot-Loop Memory Pressure
- **Pattern**: `libs/keyforge-physics/src/analysis/heuristics.rs` performs `.clone()` on `EvaluationContext` and `Geometry` inside analysis loops.
- **Pattern**: `libs/keyforge-physics/src/kernel/stages/corpus.rs` clones the entire `char_freqs` array and bigram vectors during stage execution.
- **Debt**: Unnecessary allocations in the physics kernel increase GC/Memory pressure on high-concurrency worker nodes.

### 3.2 SIMD Implementations
- **Finding**: `score_layout_avx2` (Intel) and the ARM NEON kernels are still `unimplemented!`.
- **Debt**: We are paying for 64-bit compute power but running on a scalar fallback.

---

## 4. Maintenance Debt (Invisible Debt) 🔵

### 4.1 Magic Numbers in Core
- **Finding**: Hardcoded scoring values (`100.0`, `200.0`, `500`) used in `heuristics.rs` and `analysis/mod.rs`.
- **Debt**: Changing the physical "feel" of the optimizer requires editing multiple Rust files instead of one central `physics_constants.rs`.

---

## Remediation Roadmap (Scrubbing Order)

1.  **Safety First**: Replace all `.unwrap()` in `keyforge-physics` with `Result` types and `ForgeError` variants.
2.  **Restore Context**: Refactor `infra` and `model` to stop using `.to_string()` for error mapping.
3.  **Unified Foundation**: Synchronize all `Cargo.toml` files to `workspace = true`.
4.  **Zero-Allocation Kernels**: Refactor `physics` stages to use references instead of `.clone()`.
