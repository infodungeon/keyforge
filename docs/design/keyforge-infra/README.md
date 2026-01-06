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

## 2. Corpus Loading Pipeline

The loading of a standard corpus (e.g., `en_std`) involves a complex pipeline of caching, segmented file merging, and synthetic data injection.

```mermaid
sequenceDiagram
    participant Client as Application
    participant Cache as CachingProvider
    participant FS as FsProvider
    participant Disk as Filesystem
    participant Logic as SyntheticInjector

    Note over Client: Requesting "text/en_std"

    Client->>Cache: load_corpus([Source("text/en_std")])
    
    Cache->>Cache: Check Memory (moka)
    alt Cache Hit
        Cache-->>Client: Arc(Corpus)
    else Cache Miss
        Cache->>FS: load_corpus(sources)
        
        create participant Corpus
        FS->>Corpus: new()

        loop For each Source (en_std)
            Note right of FS: Path Resolution
            FS->>Disk: Exists? system/corpora/text/en_std
            Disk-->>FS: Yes (System Asset)
            
            loop For each Segment [1grams, 2grams, 3grams, words]
                FS->>Disk: Read 1grams.mpk.zst
                Disk-->>FS: Bytes (Zstd Compressed)
                
                FS->>FS: Decompress (Zstd) & Deserialize (MsgPack)
                
                FS->>FS: resolve_corpus_char()
                Note right of FS: Maps "\u0020" -> ' '
                
                FS->>Corpus: Merge Frequencies
            end
        end

        Note right of FS: Post-Processing

        FS->>Logic: inject_synthetic_data(Corpus, is_std=true)
        activate Logic
        Logic->>Corpus: Calculate Stats (Sentence Count)
        Logic->>Corpus: Inject '\n' (Enter) transitions
        Logic->>Corpus: Inject '\x08' (Backspace) transitions
        Logic->>Corpus: Sort N-Grams (Required for Binary Search)
        deactivate Logic

        FS->>Corpus: validate()
        alt Invalid
            Corpus-->>FS: Error
            FS-->>Cache: Error
            Cache-->>Client: Error
        else Valid
            Corpus-->>FS: Ok
            FS-->>Cache: Corpus
            Cache->>Cache: Insert into Memory
            Cache-->>Client: Arc(Corpus)
        end
    end
```

### Key Steps

1. **Cache Check:** The `CachingProvider` checks if the specific combination of sources exists in memory.
2. **Path Resolution:** The `FsProvider` looks in `data/system/corpora` for binary assets (`.mpk.zst`) or `data/user/corpora` for JSON.
3. **Segmented Loading:** The corpus is assembled from `1grams`, `2grams`, `3grams`, and `words` files.
4. **Token Resolution:** String tokens (like `"\u0020"` or `"SPACE"`) are resolved to Rust `char` primitives.
5. **Synthetic Injection:** For standard prose (`_std`), **Enter** and **Backspace** frequencies are synthetically injected based on punctuation and error rate models.
6. **Sorting:** N-gram vectors are sorted to enable O(log n) binary search lookups in the Physics engine.

## 3. Repository Pattern

Data access is hidden behind traits to allow swapping backends (e.g., SQLite -> Postgres).

* **UserRepo:** Manages User accounts and API keys.
* **JobRepo:** (Planned) Manages Job state and results.
