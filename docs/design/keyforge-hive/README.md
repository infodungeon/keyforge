# Design: KeyForge Hive

**Responsibility:** Job orchestration, API hosting, and State management.
**Tier:** 3 (The Shell)

## 1. Vertical Slice Architecture

Each feature is self-contained in `src/features/<name>`.

```mermaid
graph TD
    subgraph "Feature: Register Job"
        Handler[handler.rs]
        Logic[logic.rs]
        Model[models.rs]
    end
    
    subgraph "Shared Kernel"
        State[AppState]
        DB[UserRepo]
    end
    
    Handler --> Logic
    Logic --> Model
    Logic --> State
    State --> DB
```

## 2. The Worker Protocol (WebSocket)

```mermaid
sequenceDiagram
    participant Agent
    participant Hive
    participant Queue

    Agent->>Hive: Connect (WS /ws?node_id=X)
    Hive-->>Agent: 101 Switching Protocols
    
    loop Heartbeat
        Hive->>Agent: Ping
        Agent-->>Hive: Pong
    end
    
    Note over Hive: Job Submitted
    
    Hive->>Queue: Check Queue
    Queue-->>Hive: JobID
    
    Hive->>Agent: { type: "Job", id: "123" }
    Agent->>Hive: GET /jobs/123/config
    
    Note over Agent: Optimizing...
    
    Agent->>Hive: POST /results
    Hive-->>Agent: 200 OK
```
