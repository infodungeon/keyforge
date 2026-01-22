# Design: KeyForge Protocol

**Responsibility:** Data Transfer Objects (DTOs), API Contract, and Error Registry.
**Tier:** 2 (The Contract)
**Dependencies:** `keyforge-model`.

## 1. The Wire Contract

This crate defines the shape of data sent over HTTP and WebSockets. It acts as the **Anti-Corruption Layer** between the outside world (JSON) and the internal domain (`keyforge-model`).

### DTO Structure Map

```mermaid
classDiagram
    %% --- Client -> Server ---
    class JobRequest {
        +u32 version
        +JobConfig config (flattened)
        +validate() Result
    }

    class JobConfig {
        +KeyboardDefinition definition
        +ScoringWeights weights
        +SearchParams params
        +Vec~KeyConstraint~ pinned_keys
        +Vec~CorpusSource~ corpora
        +CostMatrixSource cost_matrix
        +Vec~BiometricSample~ biometrics
        +Option~String~ parent_job_id
        +Option~f32~ baseline_score
        +Vec~String~ parents
        +id() Result~String~
        +validate() Result
    }

    %% --- Server Responses ---
    class JobResponse {
        +String job_id
        +bool is_new
    }

    class JobQueueResponse {
        +Option~String~ job_id
        +Option~JobConfig~ config
    }

    %% --- Worker -> Server ---
    class ResultSubmission {
        +u32 version
        +String job_id
        +String layout
        +f32 score
        +u64 timestamp
        +u64 nonce
        +String node_id
        +String signature
        +validate() Result
    }

    %% --- Relationships ---
    JobRequest *-- JobConfig : contains
    JobRequest ..> JobResponse : produces
    JobConfig ..> ResultSubmission : produces
```

### Worker Node DTOs

```mermaid
classDiagram
    class NodeRequest {
        +u32 version
        +String node_id
        +String cpu_model
        +i32 cores
        +Option~i32~ l2_cache_kb
        +f32 ops_per_sec
        +Option~String~ public_key
        +validate() Result
    }

    class NodeResponse {
        +String status
        +TuningProfile tuning
    }

    class TuningProfile {
        +String strategy
        +usize batch_size
        +usize thread_count
    }

    class NodeTelemetry {
        +Option~String~ job_id
        +f32 ips
        +f32 temp
        +Option~f32~ current_best
        +u64 memory_usage
        +u64 timestamp
    }

    NodeRequest --> NodeResponse : produces
```

### Asset DTOs

```mermaid
classDiagram
    class BiometricSample {
        +String bigram
        +f64 ms
        +u64 timestamp
        +validate() Result
    }

    class AssetManifestEntry {
        +String id
        +String hash
        +u64 size_bytes
        +u64 last_updated
    }

    class UserStatsStore {
        +u64 sessions
        +u64 total_keystrokes
        +Vec~BiometricSample~ biometrics
    }

    class PopulationResponse {
        +Vec~String~ layouts
    }
```

## 2. Validation Strategy (The Bouncer Pattern)

The Protocol layer acts as the **Bouncer** for the system. It enforces "Envelope Integrity" before allowing data to reach the Domain.

### Responsibilities

1. **Protocol Validation (The Envelope):**
   - **Versioning:** Is `version` compatible?
   - **Limits:** Is the payload too large? (e.g., `biometrics.len() > MAX_BIOMETRIC_SAMPLES`)
   - **Structure:** Are required fields present?
   - **Timestamps:** Is the submission within acceptable time skew?

2. **Domain Delegation (The Payload):**
   - The DTO **must** call `.validate()` on all embedded Domain Entities
   - Ensures business rules (e.g., "Weights must be positive") are enforced at API boundary

## 3. Security & DoS Protection

To prevent memory exhaustion attacks via massive JSON payloads, we enforce strict limits on collection sizes during deserialization.

```rust
#[serde(deserialize_with = "crate::serde_utils::deserialize_limited_vec")]
pub pinned_keys: Vec<KeyConstraint>,  // Capped at 100,000 items
```

**Protected Fields:** `biometrics`, `pinned_keys`, `corpora`, `layouts`.

## 4. The Error Registry

Centralized `ErrorCode` enum for consistent error handling across the stack:

```mermaid
classDiagram
    class ErrorCode {
        <<enum>>
        InternalError
        BadRequest
        NotFound
        AuthMissing
        AuthInvalid
        AuthExpired
        AuthForbidden
        JobValidationFailed
        JobNotFound
        JobAlreadyExists
        DatabaseError
        UpstreamTimeout
        ServiceUnavailable
    }

    class ErrorResponse {
        +ErrorCode code
        +String message
        +Option~Value~ details
        +new(code, message) ErrorResponse
        +with_details(details) ErrorResponse
    }

    ErrorResponse --> ErrorCode
```

**Invariant:** Every API error maps to a specific `ErrorCode`.

## 5. Versioning

```rust
pub const PROTOCOL_VERSION: u32 = 2;
pub const MIN_CLIENT_VERSION: u32 = 1;
pub const MIN_SERVER_VERSION: u32 = 1;

pub fn check_version_compatibility(client: u32, server: u32) -> Result<(), String>;
```

Clients and Workers announce their version on connection. The Server rejects incompatible versions.

## 6. TypeScript Integration

To ensure the Frontend (`keyforge-ui`) stays in sync with the Backend:

- **Feature Flag:** `ts_bindings`
- **Mechanism:** `#[derive(TS)]` on DTOs
- **Output:** `bindings/` directory
- **Usage:** Frontend imports types directly for compile-time safety

## 7. Module Structure

```
keyforge-protocol/src/
├── assets.rs      # BiometricSample, AssetManifestEntry, UserStatsStore, PopulationResponse
├── constants.rs   # Limits (MAX_BIOMETRIC_SAMPLES, time skew thresholds)
├── error.rs       # ErrorCode, ErrorResponse
├── job.rs         # JobRequest, JobConfig, JobResponse, JobQueueResponse, ResultSubmission
├── node.rs        # NodeRequest, NodeResponse, NodeTelemetry, TuningProfile
├── serde_utils.rs # deserialize_limited_vec
├── telemetry.rs   # SystemMetrics
└── lib.rs         # Re-exports, version constants
```

## 8. Key Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `PROTOCOL_VERSION` | 2 | Current wire format version |
| `MAX_BIOMETRIC_SAMPLES` | 10,000 | DoS protection limit |
| `MAX_FUTURE_SKEW_SEC` | 300 | Timestamp validation (5 min) |
| `MAX_PAST_SKEW_SEC` | 86400 | Timestamp validation (24 hr) |
| `MAX_BIOMETRIC_MS` | 10,000 | Realistic typing latency ceiling |
