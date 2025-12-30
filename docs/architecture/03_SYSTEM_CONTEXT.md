# System Context (C4 Level 1)

**Version:** 1.0
**Context:** Crate Dependencies and Boundaries.

## Architectural Rules
1.  **Gravity:** Dependencies flow DOWN. `physics` knows nothing of `hive`.
2.  **Purity:** `physics` and `evolution` are Pure Rust (No IO).
3.  **Isolation:** `protocol` defines the contract between the System and the World.

```mermaid
graph TD
    subgraph "Drivers (The World)"
        CLI[keyforge-cli]
        HIVE[keyforge-hive]
        WASM[keyforge-wasm]
    end

    subgraph "Adapters (The Glue)"
        INFRA[keyforge-infra]
        PERSIST[keyforge-persistence]
    end

    subgraph "Ports (The Contract)"
        PROTO[keyforge-protocol]
    end

    subgraph "Core (The Nucleus)"
        EVO[keyforge-evolution]
        PHYS[keyforge-physics]
        MODEL[keyforge-model]
    end

    %% Dependencies
    CLI --> HIVE
    HIVE --> INFRA
    HIVE --> PROTO
    
    INFRA --> PERSIST
    INFRA --> EVO
    
    EVO --> PHYS
    PHYS --> MODEL
    
    PROTO --> MODEL
    
    %% Cross-Cutting
    HIVE -.-> EVO : Orchestrates
```

## Crate Responsibilities

| Crate | Tier | Responsibility | Constraints |
|-------|------|----------------|-------------|
| `keyforge-model` | **Tier 1** | Domain Entities, Types, Constants. | No Logic. No IO. |
| `keyforge-physics` | **Tier 1** | Scoring Logic, Cost Calculation. | Pure Math. No `std::fs`. |
| `keyforge-evolution` | **Tier 1** | Annealing Loop, Optimization Strategy. | CPU Bound. |
| `keyforge-protocol` | **Tier 2** | DTOs, Serialization, Error Registry. | Lightweight. |
| `keyforge-infra` | **Tier 3** | Database, File System, External APIs. | Async/Tokio allowed. |
| `keyforge-hive` | **Tier 3** | HTTP Server, Job Queue, State Management. | The Application Entry Point. |
