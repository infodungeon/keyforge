# KeyForge Error Handling Architecture

**Status:** ACTIVE
**Version:** 1.0
**Context:** System-wide strategy for error propagation, mapping, and observability.

## 1. Core Principles

KeyForge adheres to the **"Semantic Truth"** doctrine. Errors are not just strings; they are structured types that represent the state of the system.

1.  **No Lazy Errors**: Use of `anyhow`, `eyre`, or `Box<dyn Error>` is strictly forbidden in library code and production binaries. Use `thiserror` to define explicit enums.
2.  **Total Propagation**: Errors must be propagated via `Result`. `panic!`, `unwrap()`, and `expect()` are forbidden in production paths (enforced by `no-unwrap-in-production` rule).
3.  **Context Preservation**: When mapping errors between layers, the original context must be preserved using structured variants, not `.map_err(|e| e.to_string())`.
4.  **Correct-by-Construction**: Favor `Typestates` and `Newtypes` to make error states unrepresentable where possible.

## 2. Error Hierarchy

The system is organized into a tiered hierarchy corresponding to the Hexagonal Architecture.

### Tier 1: Domain Errors (`ForgeError`)
- **Location**: `libs/keyforge-model/src/error.rs`
- **Purpose**: Pure logic failures (e.g., `InvalidData`, `NotFound`, `PhysicsViolation`).
- **Mapping**: The "Great Unifier". All library errors eventually map to or from `ForgeError`.

### Tier 2: Infrastructure Errors (`InfraError`)
- **Location**: `libs/keyforge-infra/src/error.rs`
- **Purpose**: I/O, Network, Database, and Serialization failures.
- **Specialization**: Includes `is_retryable()` logic for transient network/disk issues.

### Tier 3: Application Errors
- **`AgentError`**: (`apps/keyforge-agent/src/agent/errors.rs`) Handles worker-specific failures like calibration, WebSocket drops, and identity corruption.
- **`CommandError`**: (`apps/keyforge-ui/src-tauri/src/error.rs`) Standardizes errors for the React frontend, mapping everything to a machine-readable `ErrorCode`.

### Tier 4: Edge Errors (`WasmError`)
- **Location**: `libs/keyforge-wasm/src/lib.rs`
- **Purpose**: Structured JSON responses for JavaScript consumers.

## 3. Mapping Rules (The Boundary Policy)

| From | To | Strategy |
| :--- | :--- | :--- |
| `std::io::Error` | `InfraError` | `#[from]`. Wrap in `InfraError::Io`. |
| `InfraError` | `ForgeError` | Explicit `match`. `Io` becomes `ForgeError::Io`. |
| `ForgeError` | `CommandError` | Categorization. `NotFound` -> `ErrorCode::NotFound`. |
| `sqlx::Error` | `AppError` | (Hive only). Map to `AppError::Database`. |

### Example: The 100x Way
```rust
// GOOD: Structured mapping
impl From<InfraError> for ForgeError {
    fn from(e: InfraError) -> Self {
        match e {
            InfraError::Io(io) => ForgeError::Io(io.to_string()),
            InfraError::Config(s) => ForgeError::Config(s),
            // ...
        }
    }
}

// BAD: Information erasure
let res = call_infra().map_err(|e| ForgeError::Internal(e.to_string()))?;
```

## 4. Observability & Debugging

- **Logging**: Errors are automatically logged with full stack traces when using the `instrument` attribute from `tracing`.
- **User Feedback**: The CLI and UI must display human-readable advice for actionable errors (e.g., "Check your internet connection" for `InfraError::Network`).
- **Telemetry**: Error counts are aggregated in `Valkey` to detect cluster-wide hotspots or "Poison Pill" jobs.

## 5. Checklist for Developers

- [ ] Does my new error type implement `std::fmt::Display` and `std::error::Error` (via `thiserror`)?
- [ ] Have I avoided `unwrap()` and `expect()`?
- [ ] When calling an external library, have I wrapped its error in a local enum variant?
- [ ] If the error is returned to the UI, is there a corresponding `ErrorCode` in `keyforge-protocol`?
- [ ] Have I updated the relevant `From` implementations in `keyforge-ui/src-tauri/src/error.rs`?
