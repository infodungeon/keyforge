# UI Data Interaction Lifecycle

This diagram illustrates the data flow across the distributed system, explicitly differentiating between the **Local Workspace** (Client) and the **Remote Authority** (Server).

**Key Distinctions:**

1. **Local Data:** Used for UI rendering, configuration, and local storage of custom layouts.
2. **Server Data:** Split into **System** (Authority) and **User** (Ingestion).

```mermaid
sequenceDiagram
    autonumber
    actor User
    
    box "Client Machine (Local)" #f9f9f9
        participant UI as KeyForge UI
        participant LocalCore as Tauri Backend
        participant LocalFS as Local /data
    end
    
    box "Server Machine (Remote)" #e1f5fe
        participant Hive as Hive API
        participant Cache as GlobalAssetCache
        participant SvrSys as Server /data/system
        participant SvrUser as Server /data/user
    end

    Note over UI, LocalFS: 🟢 PHASE 1: BOOTSTRAP (Local Read)
    
    User->>UI: Launch App
    UI->>LocalCore: cmd_get_default_config()
    
    LocalCore->>LocalFS: Check user/config/config.json
    alt User Override Exists
        LocalFS-->>LocalCore: Return User Config
    else Default
        LocalCore->>LocalFS: Read system/config/config.json
        LocalFS-->>LocalCore: Return System Config
    end
    LocalCore-->>UI: Config Loaded

    Note over UI, Cache: 🔵 PHASE 2: SYNCHRONIZATION (Remote RAM Read)
    
    User->>UI: Click "Refresh Data"
    UI->>LocalCore: cmd_sync_data()
    LocalCore->>Hive: GET /manifest
    
    Note right of Hive: Manifest served from Memory
    Hive->>Cache: get_manifest()
    Cache-->>Hive: ServerManifest (Hashes)
    Hive-->>LocalCore: ServerManifest
    
    loop Download Assets
        LocalCore->>Hive: GET /data/system/...
        Hive->>Cache: get_file_content(...)
        Cache-->>Hive: Bytes
        Hive-->>LocalCore: Bytes
        LocalCore->>LocalFS: WRITE system/... (Update Replica)
    end

    Note over UI, LocalFS: 🟠 PHASE 3: EDITING (Local Write)
    
    User->>UI: Save "My Layout"
    UI->>LocalCore: cmd_save_user_layout()
    
    Note right of LocalCore: Writes ALWAYS target Local User
    LocalCore->>LocalFS: WRITE user/keyboards/my_layout.json
    LocalFS-->>LocalCore: Success
    LocalCore-->>UI: Saved

    Note over UI, SvrUser: 🟣 PHASE 4: OPTIMIZATION (Remote Ingestion)
    
    User->>UI: Click "Start Optimization"
    UI->>Hive: POST /jobs (JobRequest)
    
    rect rgb(240, 248, 255)
        note right of Hive: Validation hits Memory Cache
        Hive->>Cache: load_cost_matrix("cost.json")
        Cache-->>Hive: Arc<RawCostData>
        Hive->>Cache: load_corpus("en_std")
        Cache-->>Hive: Arc<Corpus>
    end
    
    rect rgb(255, 240, 240)
        note right of Hive: Persistence hits Ingestion Disk
        Hive->>SvrUser: WRITE queue/{uuid}.bin (WAL)
        SvrUser-->>Hive: Acknowledged
    end
    
    Hive-->>UI: Job ID (Active)
```

## Directory Roles & Locations

| Context | Path | Role | Access Pattern | Content |
| :--- | :--- | :--- | :--- | :--- |
| **Client** | `/data/system` | **Replica** | **Read-Only** | Synced copy of standard assets (keyboards, corpora). |
| **Client** | `/data/user` | **Workspace** | **Read-Write** | User-created layouts, uploaded corpora, temporary job queues, logs. |
| **Server** | `/data/system` | **Authority** | **Read-Only** | The canonical source of truth. Accessed via **RAM** (Cache) for Sync & Compute. |
| **Server** | `/data/user` | **Ingestion** | **Write-Only** | Incoming Job Queue (`user/queue`) and Dead Letter Queue (`user/dlq`). |
