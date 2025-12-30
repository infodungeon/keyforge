# Full System Startup Sequence

This diagram illustrates the "Eager Loading" phase where **all** JSON assets are read from disk and cached in RAM before the server accepts connections.

**Entry Point:** `crates/keyforge-hive/src/main.rs` $\rightarrow$ `state.assets.warm_all()`

```mermaid
sequenceDiagram
    autonumber
    participant Main as Hive Main
    participant Cache as GlobalAssetCache
    participant Provider as FsProvider (Infra)
    participant Disk as FileSystem

    Note over Main, Disk: Phase 1: Keyboards
    
    Main->>Cache: warm_all()
    
    Cache->>Provider: list_keyboards()
    Provider->>Disk: Scan /data/keyboards/*.json
    Disk-->>Provider: ["corne", "ansi_104", ...]
    
    loop Every Keyboard
        Cache->>Provider: load_keyboard("corne")
        Provider->>Disk: Read JSON
        Note right of Provider: Parse to KeyboardDefinition
        Provider-->>Cache: Return Arc(Keyboard)
        Cache->>Cache: Insert into RAM
    end

    Note over Main, Disk: Phase 2: Corpora

    Cache->>Provider: list_corpora()
    Provider->>Disk: Scan /data/corpora/*
    Disk-->>Provider: ["text/en_std", "code/rust", ...]

    loop Every Corpus
        Cache->>Provider: load_corpus(id)
        Provider->>Disk: Read 1grams, 2grams, etc.
        Note right of Provider: Parse & Merge Frequencies
        Provider-->>Cache: Return Arc(Corpus)
        Cache->>Cache: Insert into RAM
    end

    Note over Main, Disk: Phase 3: Physics Data

    Cache->>Provider: list_cost_matrices()
    Provider->>Disk: Scan /data/*cost*.json
    Disk-->>Provider: ["cost_matrix.json", ...]

    loop Every Cost Matrix
        Cache->>Provider: load_cost_matrix()
        Provider->>Disk: Read JSON
        Provider-->>Cache: Return Arc(RawCostData)
        Cache->>Cache: Insert into RAM
    end

    Note over Main, Disk: Phase 4: System Config

    Cache->>Provider: load_keycodes("keycodes.json")
    Provider->>Disk: Read JSON
    Provider-->>Cache: Return Arc(KeycodeRegistry)
    
    Cache->>Provider: load_app_config("config.json")
    Provider->>Disk: Read JSON
    Provider-->>Cache: Return Arc(Config)

    Cache-->>Main: Result::Ok
    
    Note right of Main: 🚀 Server Ready.<br/>Disk is now idle.
