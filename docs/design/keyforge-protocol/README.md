# Design: KeyForge Protocol

**Responsibility:** Data Transfer Objects (DTOs), API Contract, and Error Registry.
**Tier:** 2 (The Contract)
**Dependencies:** `keyforge-model`.

## 1. The Wire Contract

This crate defines the shape of data sent over HTTP and WebSockets. It acts as the **Anti-Corruption Layer** between the outside world (JSON) and the internal domain (`keyforge-model`).

### Job Lifecycle DTOs

| DTO | Direction | Purpose |
| :--- | :--- | :--- |
| **`JobRequest`** | Client -> Server | Request a new optimization job. Contains `KeyboardDefinition`, `CorpusSource`, etc. |
| **`JobConfig`** | Server -> Worker | The fully resolved configuration for a worker to execute. |
| **`ResultSubmission`** | Worker -> Server | The optimized layout and its score, signed by the worker. |

## 2. The Error Registry

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

## 3. TypeScript Integration

To ensure the Frontend (`keyforge-ui`) stays in sync with the Backend, we use `ts-rs` to generate TypeScript definitions from Rust structs.

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

## 4. Versioning

* **`PROTOCOL_VERSION`**: A monotonic integer incremented whenever the wire format changes in a breaking way.
* **Handshake:** Clients and Workers must announce their version on connection. The Server rejects incompatible versions.
