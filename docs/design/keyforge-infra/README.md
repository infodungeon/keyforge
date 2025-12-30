# Design: KeyForge Infra

**Responsibility:** IO Adapters (Filesystem, Network, Database).
**Tier:** 3 (The Adapter)

## 1. Asset Management

The `AssetManager` abstracts the difference between local files and remote resources.

```mermaid
sequenceDiagram
    participant App
    participant Manager as AssetManager
    participant FS as FsProvider
    participant Net as HiveClient

    App->>Manager: get("corpus/english.json")
    
    Manager->>FS: exists?
    alt Yes
        FS-->>Manager: Path
    else No
        Manager->>Net: download()
        Net-->>Manager: Stream
        Manager->>FS: write_atomic()
        FS-->>Manager: Path
    end
    
    Manager-->>App: Path
```

## 2. Repository Pattern

Data access is hidden behind traits to allow swapping backends (e.g., SQLite -> Postgres).

* **UserRepo:** Manages User accounts and API keys.
* **JobRepo:** (Planned) Manages Job state and results.
