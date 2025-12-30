# KeyForge CLI Sequence Diagram

This diagram illustrates the execution flow of the `keyforge search` command, demonstrating the interaction between the CLI, Infrastructure, Workspace, and Compute layers.

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant CLI as CLI (Main)
    participant Builder as SessionBuilder (Workspace)
    participant Infra as FsProvider (Infra)
    participant Evo as Evolution (Optimizer)

    Note over User, Evo: Phase 1: Initialization & Loading

    User->>CLI: keyforge search --keyboard corne --corpus text/en_std
    
    CLI->>CLI: Parse Arguments (Clap)
    CLI->>CLI: Resolve Data Root
    
    CLI->>Builder: build_session(args)
    activate Builder
    
    Builder->>Infra: new(root)
    
    par Load Assets
        Builder->>Infra: load_keyboard("corne")
        Infra-->>Builder: KeyboardDefinition
        Builder->>Infra: load_corpus("text/en_std")
        Infra-->>Builder: Corpus
        Builder->>Infra: load_cost_matrix("cost.json")
        Infra-->>Builder: RawCostData
    end
    
    Builder->>Builder: Convert to Domain Models (FixedPoint)
    
    create participant Engine as ScoringEngine (Physics)
    Builder->>Engine: new(Keyboard, Corpus, Rubric)
    activate Engine
    Engine-->>Builder: Arc<ScoringEngine>
    deactivate Engine

    create participant Runtime as Runtime (Compute)
    Builder->>Runtime: new(Engine, Registry, Config)
    
    create participant Session as Session (Workspace)
    Builder->>Session: new(State, Runtime)
    
    Builder-->>CLI: Session
    deactivate Builder

    Note over User, Evo: Phase 2: Optimization Loop

    CLI->>Session: optimize_with_callback(CliProgress)
    activate Session
    
    Session->>Runtime: optimize(callback)
    activate Runtime
    
    Runtime->>Evo: evolve(Engine, Config, callback)
    activate Evo
    
    loop Simulated Annealing
        Evo->>Engine: score(layout)
        Evo->>CLI: callback.on_progress()
        CLI-->>User: Update Progress Bar
    end
    
    Evo-->>Runtime: OptimizationResult
    deactivate Evo
    
    Runtime-->>Session: OptimizationResult
    deactivate Runtime
    
    Session-->>CLI: OptimizationResult
    deactivate Session

    CLI->>User: Print Final Layout & Score
```
