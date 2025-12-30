# Data Flow & Sequences

**Version:** 4.0
**Context:** Runtime behavior, including Failure Modes.

## 1. Job Submission (Happy Path)

```mermaid
sequenceDiagram
    participant User
    participant Hive
    participant DB

    User->>Hive: POST /jobs
    Hive->>Hive: Validate Schema
    Hive->>DB: Insert Job (Pending)
    DB-->>Hive: JobID
    Hive-->>User: 201 Created
```

## 2. Worker Execution (Happy Path)

```mermaid
sequenceDiagram
    participant Worker
    participant Hive
    participant DB

    Worker->>Hive: GET /jobs/queue
    Hive->>DB: Fetch Pending
    DB-->>Hive: Job Config
    Hive-->>Worker: Job Payload
    
    Note over Worker: Optimization Loop
    
    Worker->>Hive: POST /results
    Hive->>DB: Update Job (Completed)
    Hive-->>Worker: 200 OK
```

## 3. Failure Mode: Worker Crash (Heartbeat Timeout)

What happens when a worker pulls a job but dies before finishing?

```mermaid
sequenceDiagram
    participant Monitor as Hive::Monitor
    participant DB
    participant Queue

    loop Every 60s
        Monitor->>DB: Scan for Stale Jobs
        Note right of Monitor: "Running" > 5 mins ago<br/>AND No Heartbeat
        
        DB-->>Monitor: List[Job 123]
        
        Monitor->>DB: Increment Retry Count
        
        alt Retries < 3
            Monitor->>DB: Set Status = Pending
            Monitor->>Queue: Re-queue Job 123
        else Retries >= 3
            Monitor->>DB: Set Status = Failed
            Monitor->>DB: Log Error "Max Retries Exceeded"
        end
    end
```

## 4. Failure Mode: Database Timeout

What happens when the DB is under load?

```mermaid
sequenceDiagram
    participant API as Hive::Handler
    participant DB

    API->>DB: Insert Job
    
    alt Connection Timeout
        DB--xAPI: Error (Timeout)
        API->>API: Retry (Exp Backoff)
        
        alt Retry Success
            DB-->>API: JobID
        else Retry Exhausted
            API-->>User: 503 Service Unavailable
        end
    end
```
