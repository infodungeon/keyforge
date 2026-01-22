# Design: KeyForge Runner

**Responsibility:** High-level optimization orchestration for distributed workers.
**Tier:** 2 (The Glue)

## 1. Overview

`keyforge-runner` provides the `OptimizationRunner` and `Runner` abstractions that wire together compute, core, and adapter layers to execute optimization jobs. It acts as the bridge between job configuration (from `keyforge-protocol`) and the actual scoring/evolution machinery.

```mermaid
classDiagram
    class OptimizationRunner {
        +prepare_session(loader, config, options) ScoringSession
        +run(session, job_id, stop_flag, callback, options, config) OptimizationResult
    }

    class Runner~L: AssetLoader~ {
        -loader: &L
        +new(loader) Runner
        +prepare_job(config, keycodes_file) Runtime
    }

    class RunnerOptions {
        +timeout_sec: u64
        +log_sampling_rate: usize
        +keycodes_file: String
        +seed: Option~u64~
        +threads: usize
    }

    OptimizationRunner ..> ScoringSession : creates
    Runner ..> Runtime : creates
```

## 2. Session Preparation Flow

The runner loads all required assets (corpus, cost matrix, keycodes) and constructs a `ScoringSession` ready for optimization.

```mermaid
sequenceDiagram
    participant Agent
    participant Runner as OptimizationRunner
    participant Builder as SessionBuilder
    participant Loader as AssetLoader
    participant Adapter

    Agent->>Runner: prepare_session(loader, config, options)
    Runner->>Builder: new(loader)
    Runner->>Builder: with_keyboard_def()
    Runner->>Builder: with_corpus()
    Builder->>Loader: load corpus
    Loader-->>Builder: corpus data
    Runner->>Builder: with_cost_matrix()
    Builder->>Loader: load cost matrix
    Loader-->>Builder: cost matrix
    Runner->>Builder: with_keycodes()
    Runner->>Adapter: to_domain_rubric(weights)
    Adapter-->>Runner: Rubric
    Runner->>Builder: build()
    Builder-->>Runner: ScoringSession
```

## 3. Optimization Execution

Once prepared, the runner spawns a blocking task to execute the optimization loop.

```mermaid
sequenceDiagram
    participant Agent
    participant Runner as OptimizationRunner
    participant Tokio as spawn_blocking
    participant Core as keyforge_core

    Agent->>Runner: run(session, job_id, callback, ...)
    Runner->>Runner: resolve pinned keys
    Runner->>Tokio: spawn_blocking
    Tokio->>Core: optimize_with_engine(engine, config, callback)
    Core-->>Tokio: OptimizationResult
    Tokio-->>Runner: Result
    Runner-->>Agent: OptimizationResult
```

## 4. Dependencies

| Crate | Purpose |
|-------|---------|
| `keyforge-compute` | `SessionBuilder`, `Runtime` |
| `keyforge-core` | `ScoringSession`, `optimize_with_engine` |
| `keyforge-adapter` | `to_domain_rubric` conversion |
| `keyforge-protocol` | `JobConfig` DTO |
| `keyforge-model` | Domain types (`KeyCode`, `OptimizationResult`) |
| `keyforge-infra` | Asset loading infrastructure |
