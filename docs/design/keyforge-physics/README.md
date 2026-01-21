# Design: KeyForge Physics

**Responsibility:** Pure mathematical scoring of keyboard layouts.
**Tier:** 1 (The Nucleus)

## 1. The Scoring Engine (Multi-Tiered)

The `ScoringEngine` trait defines the interface for evaluating keyboard layouts. It is a read-only component optimized for O(1) lookups. Multiple engine implementations exist for different performance/accuracy trade-offs.

### Engine Implementations

| Engine | Purpose | Capabilities |
|--------|---------|--------------|
| `GenericScoringEngine` | Default optimized engine | Fast, portable |
| `ExactScoringEngine` | Oracle for verification | Bit-perfect, slow |
| `IntelScoringEngine` | Hardware-accelerated | AVX2, cache-aware blocking |

```mermaid
classDiagram
    class ScoringEngine {
        <<trait>>
        +name() &str
        +capabilities() EngineCapabilities
        +key_count() usize
        +score(Layout) Score
        +score_detailed(Layout) (i64, i64, i64)
        +calculate_swap_delta(Layout, pos_map, idx_a, idx_b) i64
        +analyze(Layout) AnalysisReport
        +suggest_improvements(Layout, include_thumbs) Vec~SwapSuggestion~
        +context() &EngineContext
    }

    class EngineCapabilities {
        +is_exact: bool
        +supports_avx2: bool
        +supports_blocking: bool
    }

    class GenericScoringEngine
    class ExactScoringEngine
    class IntelScoringEngine

    ScoringEngine <|.. GenericScoringEngine
    ScoringEngine <|.. ExactScoringEngine
    ScoringEngine <|.. IntelScoringEngine
    ScoringEngine --> EngineCapabilities
```

### Compilation Process

```mermaid
sequenceDiagram
    participant User
    participant Factory as EngineFactory
    participant Compiler
    participant Context as EngineContext
    participant Engine as dyn ScoringEngine

    User->>Factory: new_generic(Keyboard, Corpus, Rubric, CostModel)
    Factory->>Compiler: compile(Keyboard, Corpus, Rubric, CostModel)
    
    Compiler->>Compiler: Flatten Corpus (Bigrams -> Vec)
    Compiler->>Compiler: Pre-calculate Key Distances
    Compiler->>Compiler: Apply Cost Overrides
    
    Compiler-->>Context: EngineContext (Lookup Tables)
    
    Factory->>Engine: new(Context)
    Engine-->>User: Box<dyn ScoringEngine>
```

## 2. The Oracle Pattern (Verification)

To ensure the optimized engines remain mathematically sound despite aggressive optimizations (flattened lookups, bit-shifting, and integer scaling), we employ a "Shadow Execution" strategy. Every property test compares the high-performance engine against the `ExactScoringEngine`.

### Oracle Parity Sequence

```mermaid
sequenceDiagram
    participant T as Test Runner (Proptest)
    participant F as EngineFactory
    participant G as GenericScoringEngine
    participant E as ExactScoringEngine (Oracle)

    Note over T: Generate Random Inputs<br/>(Keyboard, Corpus, Rubric, CostModel, Layout)

    T->>F: new_generic(kb, cp, rb, cm)
    F-->>G: Box<dyn ScoringEngine>
    T->>F: new_exact(kb, cp, rb, cm)
    F-->>E: Box<dyn ScoringEngine>
    
    par Optimized Path
        T->>G: score(Layout)
        G->>G: Fixed-Point Accumulation<br/>(O(1) Lookup Tables)
        G-->>T: Result A (Score)
    and Exact Path (Oracle)
        T->>E: score(Layout)
        E->>E: Naive Iteration<br/>(Optimal Choice Search)
        E-->>T: Result B (Score)
    end

    T->>T: Assert Result A == Result B
    Note over T: Bit-for-bit parity check
```

## 3. Detailed Scoring Logic (Optimal Choice)

The engine assumes the user is an **"Optimal Typist."** For layouts with duplicate keys (e.g., bilateral spacebars or experimental layer mappings), the engine dynamically selects the physical key (or sequence of keys) that minimizes the total cost for every monogram, bigram, and trigram.

This logic ensures that adding redundant keys always improves or maintains the score, never degrades it, by finding the mathematical lower bound of effort for the given layout.

### Dynamic Search Sequence

```mermaid
sequenceDiagram
    participant S as score_layout
    participant PM as PosMap (Internal)
    participant CTX as EngineContext

    S->>PM: new(layout)
    Note right of PM: Maps KeyCode -> List of physical indices
    PM-->>S: pm

    Note over S: 1. Monogram Scoring
    loop For each char in char_freqs
        S->>PM: get(char)
        PM-->>S: candidates [p1, p2, ...]
        loop For each p in candidates
            S->>CTX: key_costs[p]
            S->>S: min_cost = min(cost, min_cost)
        end
        S->>S: total += min_cost * freq
    end

    Note over S: 2. Bigram Scoring
    loop For each (c1, c2) in bigrams
        S->>PM: get(c1)
        PM-->>S: candidates1 [p1a, p1b, ...]
        S->>PM: get(c2)
        PM-->>S: candidates2 [p2a, p2b, ...]
        loop For each p1 in candidates1
            loop For each p2 in candidates2
                S->>CTX: cost_matrix[p1, p2]
                S->>S: min_cost = min(cost, min_cost)
            end
        end
        S->>S: total += min_cost * freq
    end

    Note over S: 3. Trigram Scoring
    loop For each (c1, c2, c3) in trigrams
        S->>PM: get(c1), get(c2), get(c3)
        PM-->>S: candidates1, candidates2, candidates3
        loop For each p1 in candidates1
            loop For each p2 in candidates2
                loop For each p3 in candidates3
                    S->>S: calculate_flow_cost(p1, p2, p3)
                    S->>S: min_cost = min(cost, min_cost)
                end
            end
        end
        S->>S: total += min_cost * freq
    end
```

## 4. Public API

The crate exposes these top-level functions for one-off operations:

| Function | Purpose |
|----------|---------|
| `score(EngineRequest)` | Score a layout |
| `analyze(EngineRequest)` | Generate detailed `AnalysisReport` |
| `identify(Layout)` | Match layout to known standards (Qwerty, Dvorak, etc.) |
| `suggest_improvements(EngineRequest)` | Get swap suggestions filtered by pinned keys |

## 5. Module Structure

```
keyforge-physics/src/
├── engines/
│   ├── exact.rs          # ExactScoringEngine (Oracle)
│   ├── generic.rs        # GenericScoringEngine (Default)
│   ├── intel_comet_lake.rs # IntelScoringEngine (AVX2)
│   └── mod.rs            # ScoringEngine trait, EngineCapabilities
├── kernel/
│   ├── compute/          # Hot-path scoring logic
│   ├── stages/           # Compilation pipeline stages
│   ├── compiler.rs       # Transforms domain -> EngineContext
│   ├── mechanics.rs      # Physical key mechanics
│   └── types.rs          # ValidatedLayout, internal types
├── analysis/
│   ├── fingerprint.rs    # Layout identification (Hamming distance)
│   └── heuristics.rs     # Swap suggestion algorithms
├── verify.rs             # Layout validation
├── error.rs              # PhysicsError
└── lib.rs                # EngineFactory, public API
```
