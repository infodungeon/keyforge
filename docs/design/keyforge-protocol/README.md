# Design: KeyForge Protocol

**Responsibility:** Data Transfer Objects (DTOs), API Contract, and Error Registry.
**Tier:** 2 (The Contract)
**Dependencies:** `keyforge-model`.

## 1. The Wire Contract

This crate defines the shape of data sent over HTTP and WebSockets. It acts as the **Anti-Corruption Layer** between the outside world (JSON) and the internal domain (`keyforge-model`).

### DTO Structure Map

This diagram shows the Data Transfer Objects used to communicate between the Client, Server (Hive), and Worker (Agent).

```mermaid
classDiagram
    %% --- Client -> Server ---
    class JobRequest {
        +u32 version
        +KeyboardDefinition definition
        +ScoringWeights weights
        +SearchParams params
        +Vec~KeyConstraint~ pinned_keys
        +Vec~CorpusSource~ corpora
        +validate() Result
    }

    %% --- Server -> Worker ---
    class JobConfig {
        +KeyboardDefinition definition
        +ScoringWeights weights
        +SearchParams params
        +Vec~KeyConstraint~ pinned_keys
        +Vec~CorpusSource~ corpora
        +Option~f32~ baseline_score
    }

    %% --- Worker -> Server ---
    class ResultSubmission {
        +String job_id
        +String node_id
        +String layout
        +f32 score
        +u64 nonce
        +String signature
        +validate() Result
    }

    class NodeRequest {
        +String node_id
        +String cpu_model
        +i32 cores
        +f32 ops_per_sec
        +validate() Result
    }

    %% --- Relationships ---
    JobRequest ..> JobConfig : Transformed by Hive
    JobConfig ..> ResultSubmission : Produces
```

### Job Lifecycle DTOs

| DTO | Direction | Purpose |
| :--- | :--- | :--- |
| **`JobRequest`** | Client -> Server | Request a new optimization job. Contains `KeyboardDefinition`, `CorpusSource`, etc. |
| **`JobConfig`** | Server -> Worker | The fully resolved configuration for a worker to execute. |
| **`ResultSubmission`** | Worker -> Server | The optimized layout and its score, signed by the worker. |

## 2. Protocol Behavior

The interaction flow between the system components.

```mermaid
sequenceDiagram
    participant Client
    participant Hive as Server
    participant DB as Database
    participant Worker

    %% --- Submission Phase ---
    Note over Client, DB: Phase 1: Submission
    Client->>Hive: POST /jobs (JobRequest)
    activate Hive
    Hive->>Hive: JobRequest.validate()
    alt Invalid
        Hive-->>Client: 400 Bad Request (ErrorCode)
    else Valid
        Hive->>DB: Insert Job (Pending)
        DB-->>Hive: JobID
        Hive-->>Client: 202 Accepted (JobID)
    end
    deactivate Hive

    %% --- Execution Phase ---
    Note over Hive, Worker: Phase 2: Execution
    loop Polling
        Worker->>Hive: GET /jobs/queue (NodeRequest)
        activate Hive
        Hive->>DB: Fetch Pending Job
        alt Empty
            Hive-->>Worker: 204 No Content
        else Found
            DB-->>Hive: Job Data
            Hive->>Hive: Transform -> JobConfig
            Hive-->>Worker: 200 OK (JobConfig)
        end
        deactivate Hive
    end

    %% --- Completion Phase ---
    Note over Worker, DB: Phase 3: Completion
    Worker->>Worker: Optimize...
    Worker->>Worker: Sign Result
    Worker->>Hive: POST /results (ResultSubmission)
    activate Hive
    Hive->>Hive: ResultSubmission.validate()
    Hive->>Hive: Verify Signature
    Hive->>DB: Update Job (Completed)
    Hive-->>Worker: 200 OK
    deactivate Hive
```

## 3. Security & Validation

### DoS Protection

To prevent memory exhaustion attacks via massive JSON payloads, we enforce strict limits on collection sizes during deserialization.

* **Mechanism:** `#[serde(deserialize_with = "keyforge_model::serde_utils::deserialize_limited_vec")]`
* **Limit:** Vectors are capped at 100,000 items.
* **Target:** `biometrics`, `pinned_keys`, `corpora`.

### Validation

All DTOs implement the `Validator` trait. Validation occurs immediately upon deserialization at the API boundary.

## 4. The Error Registry

We use a centralized `ErrorCode` enum to ensure consistent error handling across the stack (Rust Backend -> TypeScript Frontend).

```rust
#[derive(strum::Display, EnumString)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Domain Errors
    JOB_VALIDATION_FAILED,
    PHYSICS_VIOLATION,
    
    // Infra Errors
    DATABASE_ERROR,
    AUTH_INVALID,
}
```

**Invariant:** Every error returned by the API must map to a specific `ErrorCode`.

## 5. TypeScript Integration

To ensure the Frontend (`keyforge-ui`) stays in sync with the Backend, we use `ts-rs` to generate TypeScript definitions from Rust structs.

* **Feature Flag:** `ts_bindings`. Must be enabled explicitly to generate bindings.
* **Source:** `#[derive(TS)]` on DTOs.
* **Output:** `bindings/` directory.
* **Usage:** The Frontend imports these types directly, ensuring compile-time safety across the network boundary.

### Example

**Rust:**

```rust
#[derive(TS)]
#[ts(export)]
pub struct BiometricSample {
    pub bigram: String,
    pub ms: f64,
}
```

**Generated TypeScript:**

```typescript
export type BiometricSample = {
  bigram: string;
  ms: number;
};
```

## 6. Versioning

* **`PROTOCOL_VERSION`**: A monotonic integer incremented whenever the wire format changes in a breaking way.
* **Handshake:** Clients and Workers must announce their version on connection. The Server rejects incompatible versions.
