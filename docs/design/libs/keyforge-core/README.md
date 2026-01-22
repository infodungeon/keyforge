# Design: KeyForge Core

**Responsibility:** Pure orchestration and helper functions.
**Tier:** 2 (The Glue)
**Constraint:** NO IO allowed. This crate must compile to WASM.

## 1. Overview

`keyforge-core` exists to prevent circular dependencies between `physics`, `evolution`, and `model`. It provides high-level functions that coordinate these lower-level crates without being tied to IO or protocols.

```mermaid
graph TD
    Core[keyforge-core]
    Physics[keyforge-physics]
    Evolution[keyforge-evolution]
    Model[keyforge-model]

    Core --> Physics
    Core --> Evolution
    Core --> Model
    Physics --> Model
    Evolution --> Physics
    Evolution --> Model
```

## 2. Public API

### Engine-Based Functions (Preferred)

Use these when you have a pre-compiled `ScoringEngine`:

| Function | Purpose |
|----------|---------|
| `build_engine(EngineRequest)` | Compile a new `ScoringEngine` |
| `score_with_engine(engine, layout)` | Score a layout → `f32` |
| `analyze_with_engine(engine, layout)` | Analyze a layout → `AnalysisReport` |
| `suggest_with_engine(engine, layout)` | Get swap suggestions |
| `optimize_with_engine(engine, config, callback, ...)` | Run optimization |
| `identify(layout)` | Match to known layout (Qwerty, etc.) |

### Request-Based Functions (Legacy)

These compile an engine internally from an `EngineRequest`:

| Function | Purpose |
|----------|---------|
| `score(EngineRequest)` | Score → `OptimizationResult` |
| `analyze(EngineRequest)` | Analyze → `AnalysisReport` |
| `suggest(EngineRequest)` | Suggest swaps |
| `optimize(EngineRequest)` | Optimize without callback |
| `optimize_with_callback(EngineRequest, CB)` | Optimize with progress |

## 3. ScoringSession

A consolidated environment holding a compiled engine with metadata:

```mermaid
classDiagram
    class ScoringSession {
        +Arc~dyn ScoringEngine~ engine
        +Arc~KeycodeRegistry~ registry
        +SearchConfig search_config
        +new(engine, registry, config) ScoringSession
    }

    class ScoringEngine {
        <<trait>>
    }

    class KeycodeRegistry {
        +get_code(label) Option~KeyCode~
        +get_label(code) Option~&str~
    }

    ScoringSession --> ScoringEngine
    ScoringSession --> KeycodeRegistry
```

## 4. AssetLoader Trait

The IO abstraction allowing core to remain filesystem/network agnostic:

```mermaid
classDiagram
    class AssetLoader {
        <<trait>>
        +load~T: Asset~(id: &str) LoaderResult~Arc~T~~
        +load_corpus(sources: &[CorpusSource]) LoaderResult~Arc~Corpus~~
    }

    class Asset {
        <<trait>>
        +category() AssetCategory
    }

    AssetLoader ..> Asset : loads
```

Implementations live in `keyforge-infra` (filesystem) and `keyforge-wasm` (in-memory).

## 5. Optimization Flow

```mermaid
sequenceDiagram
    participant Client
    participant Core
    participant Physics as dyn ScoringEngine
    participant Evo as Evolution

    Client->>Core: optimize_with_engine(engine, config, callback, ...)
    Core->>Evo: evolve(engine, config, callback, ...)
    
    loop Annealing Steps
        Evo->>Physics: calculate_swap_delta()
        Physics-->>Evo: delta (i64)
        Evo->>Evo: accept/reject
        
        opt Epoch Boundary
            Evo->>Client: callback.on_progress(epoch, score, layout, ips)
            Client-->>Evo: OptimizationControl
        end
    end
    
    Evo-->>Core: OptimizationResult
    Core-->>Client: Result
```

## 6. Re-exports

For convenience, `keyforge-core` re-exports commonly used types:

| From | Types |
|------|-------|
| `keyforge-evolution` | `EvolutionError`, `ProgressCallback` |
| `keyforge-physics` | `ScoringEngine`, `PhysicsError`, `EngineRequest`, `LayoutIdentity`, `DeterministicScorer` |

## 7. Module Structure

```
keyforge-core/src/
├── loader.rs    # AssetLoader trait, LoaderResult
├── session.rs   # ScoringSession
└── lib.rs       # Public API functions, re-exports
```
