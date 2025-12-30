# Design: KeyForge Persistence

**Responsibility:** Project state management and compilation.
**Tier:** 3 (The Adapter)

## 1. The Compiler Pattern

We distinguish between the **Stored State** (`Project`) and the **Executable State** (`Runtime`).

* **Project:** JSON-serializable DTO. Contains paths ("./data/corpus.json") and settings.
* **Runtime:** In-memory object graph. Contains loaded data (`Vec<Bigram>`) and lookup tables.

```mermaid
sequenceDiagram
    participant CLI
    participant Compiler
    participant Loader as AssetLoader
    participant Runtime

    CLI->>Compiler: compile(Project)
    
    Compiler->>Loader: load_keyboard(path)
    Loader-->>Compiler: KeyboardDefinition
    
    Compiler->>Loader: load_corpus(path)
    Loader-->>Compiler: Corpus
    
    Compiler->>Compiler: build_engine()
    
    Compiler-->>Runtime: Runtime
```

## 2. Autosave Service

Background service to persist the user's session state.

* **Strategy:** Debounced writes.
* **Format:** JSON Snapshot.
