# Design: KeyForge Compute

**Responsibility:** Application Service for local execution.
**Tier:** 2 (The Glue)

## 1. The Runtime Aggregate

The `Runtime` struct is the primary entry point for clients (CLI, GUI) that want to execute physics operations. It bundles the stateless `ScoringEngine` with the stateful `SearchConfig` and `KeycodeRegistry`.

```mermaid
classDiagram
    class Runtime {
        +Arc~dyn ScoringEngine~ engine
        +Arc~KeycodeRegistry~ registry
        +SearchConfig search_config
        +score(Layout)
        +optimize(Callback)
    }

    class ScoringEngine {
        <<interface>>
        +score(Layout) Result~Score~
        +analyze(Layout) Result~AnalysisReport~
    }

    Runtime *-- ScoringEngine : Wraps
```

## 2. Session Builder

The `SessionBuilder` (or `Compiler` in persistence) constructs a `Runtime` from raw inputs.

```mermaid
sequenceDiagram
    participant Client
    participant Builder as SessionBuilder
    participant Core as Core::Loader
    participant Runtime

    Client->>Builder: new()
    Client->>Builder: with_keyboard(def)
    Client->>Builder: with_corpus(data)
    
    Client->>Builder: build()
    Builder->>Core: build_engine(req)
    Core-->>Builder: Box<dyn ScoringEngine>
    
    Builder-->>Runtime: Runtime
    Runtime-->>Client: Ready
```
