# Design Patterns & Architectural Styles

**Version:** 4.3
**Context:** Key patterns used to enforce reliability and maintainability.

## 1. The Typestate Pattern (Compiler State Machine)

**Target:** `keyforge-hive`, `keyforge-agent`

We use the Type System to prevent invalid state transitions. Instead of a single struct with a status enum, we use distinct types.

* **Bad:** `struct Job { status: String, result: Option<f32> }`
* **Good:**
  * `struct PendingJob { config: Config }`
  * `struct RunningJob { start_time: Instant }`
  * `struct CompletedJob { score: f32 }`

**Benefit:** It is impossible to access the score of a pending job.

## 2. The Parameter Object (Context Structs)

**Target:** `keyforge-physics`

To prevent "Argument Swapping" in functions with many parameters of the same type (`f32`, `usize`), we group cohesive arguments into Context Structs.

```rust
pub struct ScoringContext<'a> {
    pub layout: &'a Layout,
    pub corpus: &'a Corpus,
    pub rubric: &'a Rubric,
}

// Signature: fn score(ctx: ScoringContext) -> Score
```

## 3. The Command Pattern (Reified Actions)

**Target:** `keyforge-hive`

We decouple the "Intent" (Command) from the "Execution" (Handler). This allows logic to be tested without an HTTP server.

```rust
pub enum HiveCommand {
    RegisterJob(JobRequest),
    CancelJob(String),
}
```

## 4. The Centralized Error Registry

**Target:** `keyforge-protocol`

We forbid lazy error handling (`anyhow`). All errors must be enumerated in a central registry to ensure clients can handle them programmatically.

```rust
#[derive(Error, Debug)]
pub enum ForgeError {
    #[error("Physics Violation: Score {0} is negative")]
    PhysicsViolation(f32),
    #[error("Stale Job: {0}")]
    StaleJob(String),
}
```

## 5. Vertical Slice Architecture

**Target:** `apps/keyforge-hive`

Organize code by **Feature**, not by technical layer.

* `src/features/register_job/` (Contains Handler, Logic, Models)
* `src/features/get_results/`

## 6. The Humble Object Pattern

**Target:** IO Boundaries

Strip logic out of the "Driver" (HTTP Handler, CLI Command) and move it to a pure, testable function.

## 7. The Hybrid DTO Pattern

**Target:** `keyforge-protocol` / `keyforge-model`

To avoid the boilerplate of maintaining duplicate structs (e.g., `JobRequestDto` vs `JobRequestDomain`), we embed **Domain Entities** directly inside **Data Transfer Objects**.

*   **The DTO (`JobRequest`):** Acts as the "Envelope". It handles serialization and Protocol-level validation (Versioning, Request Limits).
*   **The Domain Entity (`KeyboardDefinition`):** Acts as the "Payload". It handles Business Logic validation (Physics, Geometry).

**The Validation Chain:**
The DTO implements `Validator` and **must** delegate validation to its embedded Domain Entities.

```rust
impl Validator for JobRequest {
    fn validate(&self) -> Result<(), String> {
        // 1. Protocol Check (The Bouncer)
        if self.biometrics.len() > 1000 { return Err("Too big"); }
        
        // 2. Domain Check (The Rules)
        self.definition.validate()?; 
        
        Ok(())
    }
}
```

## 8. The Agent Runner Pattern

**Target:** `keyforge-cli`

Decouples User Intent (CLI Arguments) from Execution (Physics). The CLI acts as a Manager that prepares the environment and configuration, then spawns the Worker to do the job.

*   **Mechanism:** CLI builds `JobConfig` -> Serializes to JSON -> Spawns `keyforge-agent run job.json` -> Streams stdout to User.
*   **Benefit:** Ensures that "Local Search" and "Remote Job" run on identical physics engines.
