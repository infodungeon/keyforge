# Architecture Decision Record: Decoupling Data Models and System Architecture

**Date:** January 17, 2026
**Status:** Proposed
**Context:** The current architecture suffers from high coupling and rigidity across multiple domains (`CostModel`, `SearchParams`, `AnalysisReport`, `KeyNode`, `AssetLoader`, `Protocol`, `UI`). Modifications to data structures trigger cascading recompilations and refactors across the entire stack (`persistence`, `infra`, `physics`, `evolution`, `ui`).

## 1. The Problem: Strong Typing of Volatile Data

Currently, volatile configuration data is defined as explicit fields in Rust structs.

**Examples of Rigidity:**

1. **Physics (`CostModel`):** `pub sfb_penalty: f32`. Adding "lateral_stretch" breaks the schema.
2. **Optimization (`SearchParams`):** `pub temp_min: f32`. Adding "genetic_mutation_rate" breaks the schema.
3. **Reporting (`AnalysisReport`):** `pub sfb_ratio: f32`. Adding "pinky_stagger" breaks the schema.
4. **Geometry (`KeyNode`):** `pub hand: HandIndex`. Adding "switch_type" breaks the schema.

**Consequences:**

1. **High Coupling:** The UI and DB layers must know about specific physics parameters they don't use.
2. **Fragility:** Experimental changes require full stack refactors.
3. **Rigidity:** Users cannot define custom parameters or metrics without binary updates.

## 2. The Solution: Data-Driven Configuration (The "Parameter Map" Pattern)

We will transition from **Explicit Structs** to a **Parameter Map** pattern for volatile data. The `keyforge-model` crate will define the *container* structure, while the consuming crates (`physics`, `evolution`) will hold the *semantic* logic.

### A. Redefining the Data Models

We will introduce generic maps to capture dynamic data.

#### 1. Cost Model (Physics)

```rust
pub struct ScoringWeights {
    // Keep common fields for backward compatibility/autocomplete if desired
    pub sfb_penalty: f32,
    
    // Capture all new/experimental weights here
    #[serde(flatten)]
    pub dynamic_weights: HashMap<String, f32>,
}
```

#### 2. Search Configuration (Evolution)

```rust
pub struct SearchParams {
    // Universal parameters
    pub epochs: usize,
    pub seed: Option<u64>,
    
    // Algorithm-specific parameters (Annealing, Genetic, etc.)
    #[serde(flatten)]
    pub algo_params: HashMap<String, serde_json::Value>,
}
```

#### 3. Analysis Report (Reporting)

```rust
pub struct AnalysisReport {
    pub score: f32,
    
    // Generic map of metric names to values
    pub metrics: HashMap<String, f32>,
    
    // Detailed breakdowns (heatmaps, etc.)
    pub details: HashMap<String, serde_json::Value>,
}
```

#### 4. Geometry (Hardware)

```rust
pub struct KeyNode {
    // Physical reality (Immutable)
    pub x: f32,
    pub y: f32,
    
    // Metadata bag (Switch type, LED index, etc.)
    #[serde(flatten)]
    pub props: HashMap<String, serde_json::Value>,
}
```

### B. The Consumer as Gatekeeper

The consuming crates (`keyforge-physics`, `keyforge-evolution`) become the sole owners of parameter semantics. They define defaults internally and query the generic maps.

**Old Way (Fragile):**

```rust
let penalty = weights.sfb_penalty; // Compile error if field removed
```

**New Way (Robust):**

```rust
const DEFAULT_SFB: f32 = 100.0;
let penalty = weights.get("sfb_penalty").unwrap_or(DEFAULT_SFB);
```

## 3. Additional Architectural Flaws & Resolutions

### 1. The "Asset Loader" Tight Coupling

**Issue:** `AssetLoader` has specific methods for every asset type (`load_keyboard`, `load_corpus`). Adding a new type requires updating the trait and all implementations.
**Resolution: Generic Loader Pattern**
Use a generic method constrained by a marker trait.

```rust
trait Asset: DeserializeOwned + Serialize {
    const CATEGORY: &'static str;
}
trait AssetLoader {
    fn load<T: Asset>(&self, id: &str) -> Result<T>;
}
```

### 2. The "Explicit Protocol" Mirroring

**Issue:** `keyforge-protocol` mirrors `keyforge-model` structs. Changes in the model break the protocol and server.
**Resolution: Shared Kernel & Opaque Payloads**

1. Use `keyforge-model` types directly in the protocol where appropriate.
2. Use `serde_json::Value` for payloads that the intermediary (Hive) does not need to validate (e.g., job configuration details).

### 3. The "Hardcoded Test Data" Fragility

**Issue:** Integration tests construct complex mock objects in code. Changes to internal logic (e.g., thumb cost resolution) break these mocks.
**Resolution: Fixture-Based Testing**
Load test data from real JSON files ("Golden Files") instead of constructing them in code.

```rust
let kb: Keyboard = load_fixture("szr35");
```

### 4. The "UI-Backend" Contract Rigidity

**Issue:** Tauri commands are strongly typed to specific configuration structs. Adding a parameter requires updating Rust, TypeScript, and React.
**Resolution: Schema-Driven UI**

1. Backend exposes a schema endpoint (`GET /api/schema`).
2. Frontend generates forms dynamically based on the schema.
3. Commands accept generic `HashMap` or `Value` objects.

### 5. The "Compiler" Monolith

**Issue:** `keyforge-physics::Compiler` is a monolithic function handling geometry, costs, and n-grams.
**Resolution: Pipeline Architecture**
Break compilation into distinct, testable stages (`GeometryStage`, `CostStage`, `NgramStage`). This allows unit testing specific logic (like cost resolution) in isolation.

## 4. Runtime Safety & Performance

To mitigate the risks of dynamic typing, we must enforce strict boundaries.

### 1. Compilation Phase (Performance)

The `ScoringEngine` must **never** access the `HashMap` during the hot loop (`score()`).

* **Action:** The `Compiler` reads the dynamic map *once* at startup and bakes the values into a static, optimized `EngineContext` (using arrays or specific fields).
* **Benefit:** Zero runtime overhead for dynamic parameters.

### 2. Schema Validation (Safety)

We lose compile-time type checking, so we must add runtime validation.

* **Action:** Implement a `SchemaValidator` that checks the `HashMap` against a defined schema (types, ranges, required fields) *before* compilation starts.
* **Benefit:** Prevents silent failures or bad defaults (e.g., string passed for float).

### 3. Determinism

HashMaps have non-deterministic iteration order.

* **Action:** Use `BTreeMap` or sort keys before iterating when order matters (e.g., generating Job IDs, applying sequential penalties).
* **Benefit:** Ensures reproducible builds and consistent hashing across machines.

## 5. Implementation Strategy

We will execute this refactor in **Five Waves** to maintain system stability.

### Wave 1: The Cost Model (Immediate Priority)

* **Goal:** Allow adding new physics weights without breaking the build.
* **Action:** Refactor `ScoringWeights` to use `HashMap` (via `serde(flatten)`).
* **Impact:** `keyforge-physics` updates to read from map. `keyforge-ui` updates to render dynamic list.

### Wave 2: Test Stability

* **Goal:** Stop tests from breaking when internal logic changes.
* **Action:** Refactor integration tests to use `load_fixture` and real JSON files.

### Wave 3: Loader Cleanup

* **Goal:** Make adding new asset types easy.
* **Action:** Refactor `AssetLoader` to use the Generic pattern.

### Wave 4: Search Config & UI Flexibility

* **Goal:** Support multiple optimization algorithms and dynamic UI.
* **Action:** Refactor `SearchParams` to use `HashMap`. Implement Schema-Driven UI.

### Wave 5: Compiler Refactor

* **Goal:** Improve testability and maintainability of the physics engine.
* **Action:** Break `Compiler` into a pipeline architecture.

## 6. Benefits

1. **Zero-Code Config Changes:** Adding `"new_metric": 50.0` to JSON works immediately.
2. **Isolation:** Infrastructure layers no longer break when domain logic changes.
3. **Extensibility:** Users can share custom models with experimental parameters.
4. **UI Flexibility:** The frontend can render "Advanced Settings" dynamically based on the data provided.
5. **Test Robustness:** Tests rely on stable data fixtures rather than fragile code constructs.
6. **Performance:** Dynamic configuration is compiled into static speed at runtime.
