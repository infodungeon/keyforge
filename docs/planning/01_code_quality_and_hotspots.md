# Artifact 1: Code Quality & Hotspot Ledger

**Goal:** Identify the files that are most likely to break and hardest to fix.
**Input Data Sources:** Algorithmic Analysis (`Risk = Complexity * (Churn + 1)`).

## 1. Hotspot Map (Systemic Risk)

*The top 20 riskiest files in the repository based on algorithmic analysis.*

| Risk Score | File Path | Complexity | Churn | LOC | Primary Issue |
|---:|:---|---:|---:|---:|:---|
| ~~**6020**~~ | ~~`libs/keyforge-physics/src/kernel/compute.rs`~~ | ~~215~~ | ~~27~~ | ~~1100~~ | **REMEDIATED:** User confirmed complete. Refactored into `compute/` directory structure. |
| ~~**1890**~~ | ~~`libs/keyforge-model/src/config.rs`~~ | ~~126~~ | ~~14~~ | ~~675~~ | **REMEDIATED:** Split into domain-specific modules (`config/`). |
| ~~**1386**~~ | ~~`libs/keyforge-evolution/src/supervisor/annealing.rs`~~ | ~~77~~ | ~~17~~ | ~~547~~ | **REMEDIATED:** Extracted `ProgressReporter` and decomposed `run` loop. |
| ~~**1260**~~ | ~~`libs/keyforge-physics/src/kernel/compiler.rs`~~ | ~~70~~ | ~~17~~ | ~~573~~ | **REMEDIATED:** Decoupled stages into `stages/`. `compiler.rs` is now a 100-line orchestrator. |
| ~~**1127**~~ | ~~`apps/keyforge-agent/src/agent/network.rs`~~ | ~~49~~ | ~~22~~ | ~~325~~ | **REMEDIATED:** Modularized into `network/` directory (outbox, breaker, manager). |
| ~~**1100**~~ | ~~`libs/keyforge-infra/src/asset/fs_provider.rs`~~ | ~~50~~ | ~~21~~ | ~~287~~ | **REMEDIATED:** Extracted path logic to `resolver.rs`. `FsProvider` is now clean IO only. |
| ~~**1092**~~ | ~~`libs/keyforge-model/src/constants.rs`~~ | ~~91~~ | ~~11~~ | ~~366~~ | **REMEDIATED:** Pruned migrated constants to config modules. |
| ~~**1005**~~ | ~~`libs/keyforge-physics/src/verify.rs`~~ | ~~67~~ | ~~14~~ | ~~253~~ | **REMEDIATED:** Modularized `DeterministicScorer`. |
| ~~**966**~~ | ~~`libs/keyforge-infra/src/asset/caching_provider.rs`~~ | ~~69~~ | ~~13~~ | ~~396~~ | **REMEDIATED:** Extracted `AssetCache` module to handle storage. |
| ~~**800**~~ | ~~`libs/keyforge-protocol/src/protocol.rs`~~ | ~~50~~ | ~~15~~ | ~~401~~ | **REMEDIATED:** Split into `job`, `node`, `assets`, `telemetry` modules. |
| ~~**588**~~ | ~~`apps/keyforge-agent/src/main.rs`~~ | ~~42~~ | ~~13~~ | ~~414~~ | **REMEDIATED:** Refactored into `cmd/`, `identity.rs`, `config_loader.rs`. `main` is now a thin orchestrator. |
| ~~**549**~~ | ~~`apps/keyforge-hive/tests/concurrency.rs`~~ | ~~61~~ | ~~8~~ | ~~180~~ | **REMEDIATED:** Extracted shared setup to `common/mod.rs`. Test is now focused on logic. |
| **544** | `libs/keyforge-evolution/src/supervisor/strategies.rs` | 34 | 15 | 276 | **Strategy Churn:** Mutation strategies are being tweaked often. |
| **528** | `apps/keyforge-cli/src/cli_args/config.rs` | 48 | 10 | 335 | **CLI Config:** Duplication of config logic found in `model`. |
| **525** | `libs/keyforge-adapter/src/conversion.rs` | 35 | 14 | 266 | **Format War:** Converters are high maintenance. |
| **480** | `apps/keyforge-hive/src/infra/queue.rs` | 40 | 11 | 307 | **Concurrency:** Core queue logic. Critical for Hive stability. |
| **455** | `apps/keyforge-hive/src/infra/repositories/jobs.rs` | 35 | 12 | 599 | **DB Coupling:** Large repository file handling massive SQL queries. |
| **448** | `libs/keyforge-infra/src/cache.rs` | 64 | 6 | 413 | **Legacy Cache:** Separate from `caching_provider.rs`? Potential duplication. |
| **396** | `apps/keyforge-ui/src/components/panels/AnalyzePanel.tsx` | 44 | 8 | 368 | **UI Hotspot:** The most complex React component (excluding VisualBuilder). |
| **385** | `apps/keyforge-ui/src/api/web.ts` | 55 | 6 | 461 | **API Client:** Frontend API layer is large. Check for auto-generation opportunities. |

## 2. Dependency Audit Log

*State of external libraries.*

| Library | Current Version | Stable Version | Breaking Change Risk | Vulnerabilities (CVEs) | Action |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `sqlx` | 0.7 (Workspace) | 0.8 | Medium | 0 | Upgrade to 0.8 for better compile-time checks. |
| `react` | 18.3.1 | 18.3.1 | Low | 0 | Up to date. |
| `age` | 0.10 | 0.10 | Low | 0 | **Good:** Using standard crypto for agent identity. |

## 3. Dead Code Inventory

*Code that exists but is never called.*

| File / Function | Last Modified | Referenced By | Recommendation |
| :--- | :--- | :--- | :--- |
| `hosts/sites/keyforge/assets/javascripts/lunr/wordcut.js` | N/A | Unknown | **Vendor Bloat:** 6708 lines of unused search logic? Verify if `lunr` is actually used or if this is dead weight. |
| `jobs.rs`: `prune_stale_jobs` fallback | N/A | None (if migrations applied) | Remove the `42703` error handling once all envs are migrated. |

## Remediation Logic / Rules

1. **IF** Hotspot Score is High (>1000) **THEN** Task: *"[Refactor] Optimization pass on `compute.rs` to remove String allocations."*
2. **IF** React Re-renders High **THEN** Task: *"[Perf] Optimize `VisualBuilder` drag loop to use `requestAnimationFrame` or local refs."*
3. **IF** Config logic is brittle **THEN** Task: *"[Refactor] Replace hardcoded paths in `agent/main.rs` with `config` crate or centralized helper."*
