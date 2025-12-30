# Design: KeyForge Physics

**Responsibility:** Pure mathematical scoring of keyboard layouts.
**Tier:** 1 (The Nucleus)

## 1. The Scoring Engine

The `ScoringEngine` is a compiled, read-only struct optimized for O(1) lookups. It does not store the layout; it calculates the cost of applying a layout to a physical keyboard.

### Compilation Process

```mermaid
sequenceDiagram
    participant User
    participant Compiler
    participant Context as EngineContext
    participant Engine as ScoringEngine

    User->>Compiler: compile(Keyboard, Corpus, Rubric)
    
    Compiler->>Compiler: Flatten Corpus (Bigrams -> Vec)
    Compiler->>Compiler: Pre-calculate Key Distances
    Compiler->>Compiler: Apply Cost Overrides
    
    Compiler-->>Context: EngineContext (Lookup Tables)
    
    Compiler->>Engine: new(Context)
    Engine-->>User: ScoringEngine
```

### Scoring Loop (Hot Path)

```mermaid
sequenceDiagram
    participant Engine as ScoringEngine
    participant Layout
    participant Math as IntegerMath

    Engine->>Layout: Validate(KeyCount)
    
    loop For each Bigram in Corpus
        Engine->>Layout: Get KeyIndex for Char A
        Engine->>Layout: Get KeyIndex for Char B
        
        Engine->>Math: Lookup Cost(Index A, Index B)
        Math-->>Engine: Cost (f32)
        
        Engine->>Math: Cost * Frequency
        Math-->>Engine: Weighted Cost
        
        Engine->>Engine: Accumulate (i64)
    end
    
    Engine-->>User: Total Score
```

## 2. Invariants

1. **Integer Arithmetic:** All accumulation happens in `i64` to prevent floating-point drift. `f32` is only used for the initial cost lookup.
2. **Immutability:** The `ScoringEngine` is thread-safe (`Sync`) and never changes state.
3. **Determinism:** Given the same inputs, `score()` must return the exact same bitwise result.
