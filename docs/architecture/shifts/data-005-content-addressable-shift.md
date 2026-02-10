# Architectural Shift: Content-Addressable Job IDs (DATA-005)

**Status:** Implementation Phase 2 (Mapping)
**Track:** #151

## C4 Container Diagram: Identity Resolution Flow

```mermaid
sequenceDiagram
    participant API as Hive/CLI
    participant UC as OptimizationUseCase
    participant Loader as AssetLoader
    participant Fingerprint as common::calculate_fingerprint
    participant JobID as JobIdentifier

    API->>UC: prepare_session(request)
    
    loop For each CorpusSource
        UC->>Loader: get_hash(id)
        Loader-->>UC: content_hash (SHA-256)
        UC->>UC: Update src.hash
    end

    UC->>Fingerprint: calculate_fingerprint(sources_with_hashes)
    Fingerprint->>Fingerprint: Serialize & Hash sources
    Fingerprint-->>UC: corpora_fingerprint

    UC->>JobID: try_from_parts(..., corpora_fingerprint)
    JobID-->>UC: final_job_id
    UC-->>API: (final_job_id, session)
```

## Impact Analysis Summary

1. **libs/keyforge-infra**:
    - `ValkeyProvider::get_hash` must be reliable and linked to the actual blob content in the store.
    - `calculate_fingerprint` should potentially use a more robust canonicalization than `serde_json::to_vec`.
2. **libs/keyforge-compute**:
    - `OptimizationUseCase` currently attempts to resolve hashes but falls back silently if `loader.get_hash` fails.
    - **Requirement**: Make hash resolution mandatory for Job ID generation.
3. **libs/keyforge-adapter**:
    - `AssetLoader` trait `get_hash` needs clear documentation that it MUST be content-addressable.
    - `InMemoryLoader` needs to store and return real hashes of injected objects.

## Verification Strategy

- **Consistency Test**: Update a corpus, reload, and verify `JobIdentifier` changes.
- **Cache Miss Verification**: Ensure `CompiledEngineCache` correctly re-compiles when the Job ID shifts due to content changes.
