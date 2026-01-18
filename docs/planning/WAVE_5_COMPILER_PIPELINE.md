# Wave 5: Compiler Pipeline Refactor

**Goal:** Break the monolithic `Compiler::compile` into a testable pipeline of discrete stages.
**Context:** `docs/planning/refactor.md`

## Architecture Strategy

We will replace the single `compile` function with a `CompilationPipeline` that processes data in stages.

```rust
pub trait CompilationStage {
    type Input;
    type Output;
    fn execute(&self, input: Self::Input) -> Result<Self::Output, PhysicsError>;
}

// Stage 1: Geometry & Spatial Math (Distances, Finger Origins)
pub struct GeometryStage;

// Stage 2: Static Costs (Mapping KeyNodes to CostModel values)
pub struct CostStage;

// Stage 3: Corpus Flattening (Bigrams, Trigrams, Pruning)
pub struct CorpusStage;
```

## Work Items

### 1. Define Pipeline Abstractions
- [x] **1.1 Define `CompilationStage` trait:** Standard interface for stages.
- [x] **1.2 Define intermediate types:** Structs to pass data between stages.

### 2. Implement Stages
- [x] **2.1 Implementation `GeometryStage`:** Move distance matrix and finger origin logic here.
- [x] **2.2 Implementation `CostStage`:** Move `resolve_key_cost` and static cost logic here.
- [x] **2.3 Implementation `CorpusStage`:** Move bigram flattening and trigram pruning here.

### 3. Refactor `Compiler`
- [x] **3.1 Orchestrate Pipeline:** Update `Compiler::compile` to instantiate and run stages.
- [x] **3.2 Clean up `compiler.rs`:** Remove private helper functions moved to stages.

### 4. Unit Testing
- [ ] **4.1 Test `GeometryStage`:** Verify distance calculations in isolation.
- [ ] **4.2 Test `CostStage`:** Verify cost model lookups.
- [ ] **4.3 Test `CorpusStage`:** Verify pruning and flattening logic.

## Verification
- [ ] **Oracle Check:** Ensure `ScoringEngine` bit-for-bit parity remains intact.
- [ ] **Run Tests:** `just test`.
