# Entity Relationship Models

**Version:** 4.0
**Context:** `keyforge-model` and `keyforge-physics` Aggregates.

## 1. The Optimization Aggregate

The `EngineRequest` is the root aggregate that defines a "Job". It is composed of immutable reference data (`Arc`) and mutable configuration.

```mermaid
classDiagram
    class EngineRequest {
        +Arc~Keyboard~ keyboard
        +Arc~Corpus~ corpus
        +Arc~Rubric~ rubric
        +SearchConfig config
        +Option~Layout~ initial_layout
        +Vec~Option~KeyCode~~ pinned_keys
    }

    class Keyboard {
        +Vec~KeyNode~ keys
        +usize key_count
    }

    class KeyNode {
        +KeyIndex index
        +FingerIndex finger
        +HandIndex hand
        +f32 x
        +f32 y
    }

    class Corpus {
        +Vec~u64~ char_freqs
        +Vec~Bigram~ bigrams
        +Vec~Trigram~ trigrams
    }

    class Rubric {
        +f32 sfb_weight
        +f32 distance_weight
        +f32 roll_weight
    }

    class Layout {
        +Vec~KeyCode~ keys
    }

    EngineRequest *-- Keyboard : Shared (Arc)
    EngineRequest *-- Corpus : Shared (Arc)
    EngineRequest *-- Rubric : Shared (Arc)
    EngineRequest *-- Layout : Owns (Optional)
    Keyboard *-- KeyNode : Contains
```

## 2. The Scoring Engine (Compiler Pattern)

The `ScoringEngine` is a trait that abstracts different high-performance implementations. The `EngineFactory` is used to instantiate the appropriate engine based on requirements (Exact vs Optimized).

```mermaid
classDiagram
    class ScoringEngine {
        <<interface>>
        +score(Layout) Result~Score~
        +analyze(Layout) Result~AnalysisReport~
        +capabilities() EngineCapabilities
    }

    class GenericScoringEngine {
        +ctx EngineContext
    }

    class IntelScoringEngine {
        +ctx EngineContext
        +config IntelEngineConfig
    }

    class ExactScoringEngine {
        +scorer DeterministicScorer
    }

    class EngineFactory {
        +new_generic(Keyboard, Corpus, Rubric) ScoringEngine
        +new_intel_comet_lake(Keyboard, Corpus, Rubric, Config) ScoringEngine
        +new_exact(Keyboard, Corpus, Rubric) ScoringEngine
    }

    class EngineContext {
        +usize key_count
        +Vec~Score~ cost_matrix
        +LookupTables tables
    }

    ScoringEngine <|-- GenericScoringEngine
    ScoringEngine <|-- IntelScoringEngine
    ScoringEngine <|-- ExactScoringEngine
    
    GenericScoringEngine *-- EngineContext : Owns
    IntelScoringEngine *-- EngineContext : Owns
    EngineFactory ..> ScoringEngine : Creates
```

## 3. The Evolution Loop

The `Optimizer` orchestrates the mutation of the `Layout` using the `ScoringEngine` as an oracle.

```mermaid
sequenceDiagram
    participant Supervisor as Optimizer
    participant Engine as ScoringEngine
    participant Layout as Layout

    Supervisor->>Engine: new(Request)
    Engine-->>Supervisor: Ready

    loop Annealing Steps
        Supervisor->>Layout: Mutate (Swap Keys)
        Supervisor->>Engine: score(Layout)
        Engine-->>Supervisor: Score (Cost)
        
        alt Score Improved OR Temperature Check
            Supervisor->>Supervisor: Accept New Layout
        else
            Supervisor->>Layout: Revert Swap
        end
        
        opt Every N Steps
            Supervisor->>User: ProgressCallback
        end
    end

    Supervisor->>User: OptimizationResult
```
