# Design: KeyForge Physics

**Responsibility:** Pure mathematical scoring of keyboard layouts.
**Tier:** 1 (The Nucleus)

## 1. The Scoring Engine (Optimized)

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
