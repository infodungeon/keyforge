# Design Patterns & Architectural Styles

**Version:** 4.1
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
