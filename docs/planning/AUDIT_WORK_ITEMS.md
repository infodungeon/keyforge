# Audit Execution: Detailed Work Items

This document tracks the specific tool-level actions required to fulfill the "Zero Entropy" Audit.

## Sector 1: Strategic & Domain Debt
| ID | Audit Item | Tool Call / Action | Status |
| :--- | :--- | :--- | :--- |
| 1.1 | **Tech Choice Suitability** | `cargo tree -p keyforge-hive` & `cargo bloat --release`. Analyze if `sqlx`/`axum` overhead matches usage. | [X] Done |
| 1.2 | **Domain Divergence** | `narsil-mcp` symbol search `Layout`, `KeyNode`. Compare fields against `docs/domain/spec.md`. | [X] Done |
| 1.3 | **Zombie Projects** | `narsil-mcp` get_call_graph for the whole workspace. Identify crates with 0 incoming dependencies. | [X] Done |
| 1.4 | **Strategic Suitability** | Review `Cargo.toml` for "heavy" crates (e.g. `aws-sdk`, `tauri`) and use `narsil` to find how many functions actually use them. | [X] Done |

## Sector 2: Architectural Physics & Structural Debt
| ID | Audit Item | Tool Call / Action | Status |
| :--- | :--- | :--- | :--- |
| 2.1 | **Hexagonal Purity** | `narsil-mcp` find_path from `libs/keyforge-physics` to `std::fs`, `std::net`, or `tokio`. | [X] Done |
| 2.2 | **Layer Inversions** | `narsil-mcp` get_import_graph. Verify no imports from `keyforge-infra` inside `keyforge-model`. | [X] Done |
| 2.3 | **Coupling Gravity** | `narsil-mcp` get_function_hotspots. Identify symbols with > 20 total connections. | [X] Done |
| 2.4 | **Module Topology** | `arbor` get_project_structure (depth 4). Flag files > 500 lines. Identify "utils" sprawl. | [X] Done |
| 2.5 | **Circular Debt** | `narsil-mcp` find_circular_imports across all 13 crates. | [X] Done |

## Sector 3: Design Pattern & Semantic Debt
| ID | Audit Item | Tool Call / Action | Status |
| :--- | :--- | :--- | :--- |
| 3.1 | **Anemic vs Rich Models** | `ast-grep` scan for structs with `pub` fields and no `impl` methods containing validation logic. | [X] Done |
| 3.2 | **Refactoring for Reuse** | `narsil-mcp` find_similar_code across `apps/`. Identify shared scoring or conversion logic. | [X] Done |
| 3.3 | **Extensibility Review** | `narsil-mcp` get_call_graph for `EngineFactory`. Evaluate how new engines are registered (Static vs Dynamic). | [X] Done |
| 3.4 | **Trivial Implementations** | `ast-grep` scan for manual `impl From<X> for Y` that only maps identical field names. | [X] Done |
| 3.5 | **Interface Debt** | `narsil-mcp` find_symbol_usages for internal struct fields. Flag direct field access from external crates. | [X] Done |

## Sector 4: Type Safety & Coding Standards
| ID | Audit Item | Tool Call / Action | Status |
| :--- | :--- | :--- | :--- |
| 4.1 | **Panic Pathways** | `narsil-mcp` get_callees (transitive=true) for all `pub` functions in `libs/`. Flag paths ending in `unwrap`. | [X] Done |
| 4.2 | **Primitive Obsession** | `ast-grep` scan function signatures for `u16`, `u32`, `f32` where `KeyIndex`, `Score` etc exist. | [X] Done |
| 4.3 | **Async Contagion** | `narsil-mcp` find_symbols (symbol_type=function) in `physics`. Flag any returning `Future` or `impl Future`. | [X] Done |
| 4.4 | **Safety Audit** | `run_shell_command` `grep -r "unsafe" .`. Verify each has a `// Safety:` comment. | [X] Done |

## Sector 5: Verification & Determinism
| ID | Audit Item | Tool Call / Action | Status |
| :--- | :--- | :--- | :--- |
| 5.1 | **Mock Drift** | `run_shell_command` `grep -r "mockall" tests/`. Compare mock coverage vs integration tests. | [X] Done |
| 5.2 | **Fixture Fragility** | `run_shell_command` `grep -r ".json" tests/`. Compare against `proptest!` usage. | [X] Done |
| 5.3 | **Bit-Perfect Det.** | `ast-grep` scan `libs/keyforge-physics/src/kernel` for `f32` or `f64` accumulators. | [X] Done |

## Sector 6: Operational & Production
| ID | Audit Item | Tool Call / Action | Status |
| :--- | :--- | :--- | :--- |
| 6.1 | **Observability** | `ast-grep` scan for `info!`, `error!`, `debug!` calls. Ensure `tracing::instrument` is on all public entry points. | [X] Done |
| 6.2 | **Config Hardcoding** | `run_shell_command` `grep -rE "[0-9]{3,}"` (magic numbers) and string literals in code. | [X] Done |
| 6.3 | **Deployment Readiness** | `read_file` `ops/Dockerfile`. Check for multi-stage builds and non-root users. | [X] Done |
| 6.4 | **Error Mapping** | `read_file` `libs/keyforge-physics/src/error.rs`. Verify `thiserror` coverage for all failure modes. | [X] Done |

## Sector 7: Cognitive & Documentation
| ID | Audit Item | Tool Call / Action | Status |
| :--- | :--- | :--- | :--- |
| 7.1 | **ADR Parity** | `run_shell_command` `ls docs/architecture/decisions`. Cross-reference with `narsil` symbol list. | [X] Done |
| 7.2 | **API Documentation** | `run_shell_command` `cargo doc --no-deps`. Check for "missing documentation" warnings. | [X] Done |
| 7.3 | **Knowledge Gaps** | `arbor` get_project_structure. Identify crates without a `README.md` or internal module docs. | [X] Done |

## Sector 8: Supply Chain & Dependency
| ID | Audit Item | Tool Call / Action | Status |
| :--- | :--- | :--- | :--- |
| 8.1 | **Framework Tax** | `cargo tree`. Challenge `sqlx` binary size impact using `cargo bloat`. | [X] Done |
| 8.2 | **Zombie Dependencies** | `run_shell_command` `cargo udeps` (requires nightly/install). | [X] Done |
| 8.3 | **Binary Bloat** | `cargo bloat -n 20` to identify top 20 symbols taking up space. | [X] Done |

## Sector 9: Experience & UI Debt
| ID | Audit Item | Tool Call / Action | Status |
| :--- | :--- | :--- | :--- |
| 9.1 | **UI Logic Leakage** | `run_shell_command` `grep -r "physics" apps/keyforge-ui/src`. Ensure no physics logic in UI components. | [X] Done |
| 9.2 | **Consistency Audit** | Compare `keyforge-cli` help output vs `keyforge-ui` input fields. | [X] Done |
| 9.3 | **Latency Perception** | `ast-grep` scan for `async` functions in UI that lack loading state indicators. | [X] Done |

## Sector 10: Completeness & Maintenance
| ID | Audit Item | Tool Call / Action | Status |
| :--- | :--- | :--- | :--- |
| 10.1 | **TODO Archeology** | `run_shell_command` `grep -r "TODO" .`. Identify the oldest TODOs using `git blame`. | [X] Done |
| 10.2 | **Unimplemented Logic** | `run_shell_command` `grep -rE "todo\!|unimplemented\!" .`. | [X] Done |
| 10.3 | **Dead Code** | `narsil-mcp` find_unused_exports --repo keyforge. | [X] Done |
| 10.4 | **Churn Hotspots** | `run_shell_command` `git log --format=format: --name-only | sort | uniq -c | sort -nr | head -n 20`. | [X] Done |
