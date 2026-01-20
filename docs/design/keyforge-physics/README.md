# Design: KeyForge Physics

**Responsibility:** Pure mathematical scoring of keyboard layouts.
**Tier:** 1 (The Nucleus)

## 1. The Scoring Engine (Multi-Tiered)

The `ScoringEngine` is a trait that defines the interface for evaluating keyboard layouts. It is a read-only component optimized for O(1) lookups. It does not store the layout; it calculates the cost of applying a layout to a physical keyboard.

### Compilation Process

```mermaid
sequenceDiagram
    participant User
    participant Factory as EngineFactory
    participant Compiler
    participant Context as EngineContext
    participant Engine as dyn ScoringEngine

    User->>Factory: new_generic(Keyboard, Corpus, Rubric)
    Factory->>Compiler: compile(Keyboard, Corpus, Rubric)
    
    Compiler->>Compiler: Flatten Corpus (Bigrams -> Vec)
    Compiler->>Compiler: Pre-calculate Key Distances
    Compiler->>Compiler: Apply Cost Overrides
    
    Compiler-->>Context: EngineContext (Lookup Tables)
    
    Factory->>Engine: new(Context)
    Engine-->>User: Box<dyn ScoringEngine>
```

## 2. The Oracle Pattern (Verification)

To ensure the optimized engine remains mathematically sound despite aggressive optimizations (flattened lookups, bit-shifting, and integer scaling), we employ a "Shadow Execution" strategy. Every property test compares the high-performance engine against the `DeterministicScorer`.

### Oracle Parity Sequence

```mermaid
sequenceDiagram
    participant T as Test Runner (Proptest)
    participant F as EngineFactory
    participant E as dyn ScoringEngine (Generic)
    participant O as DeterministicScorer (Oracle)

    Note over T: Generate Random Inputs<br/>(Keyboard, Corpus, Rubric, Layout)

    T->>F: new_generic(kb, cp, rb)
    F->>E: Box<dyn ScoringEngine>
    
    par Optimized Path
        T->>E: score(Layout)
        E->>E: Fixed-Point Accumulation<br/>(O(1) Lookup Tables)
        E-->>T: Result A (i64)
    and Naive Path (Oracle)
        T->>O: score(kb, cp, rb, Layout)
        O->>O: Naive Iteration<br/>(Optimal Choice Search)
        O-->>T: Result B (i64)
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

## 4. Key Components

* **Compiler:** Transforms domain entities into `EngineContext`. It handles the heavy lifting of spatial math and corpus pruning so the scoring loop remains tight.
* **Compute Kernel:** The hot-path logic for monogram, bigram, and trigram scoring.
* **Heuristics:** Provides fast swap suggestions by calculating score deltas rather than full re-scores.
* **Fingerprinter:** Uses Hamming distance to identify known layout standards (Qwerty, Dvorak, etc.).
