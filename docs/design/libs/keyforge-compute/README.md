# Design: KeyForge Compute

**Responsibility:** Application Service for local execution.
**Tier:** 2 (The Glue)

## 1. The Runtime Aggregate

The `Runtime` struct is the primary entry point for clients (CLI, GUI) that want to execute physics operations. It bundles the stateless `ScoringEngine` with the stateful `SearchConfig` and `KeycodeRegistry`.

```mermaid
classDiagram
    class Runtime {
        +Arc~dyn ScoringEngine~ engine
        +Arc~KeycodeRegistry~ registry
        +SearchConfig search_config
        +new(engine, registry, config) Runtime
        +score(Layout) Result~f32~
        +analyze(Layout) Result~AnalysisReport~
        +suggest_improvements(Layout) Result~Vec~SwapSuggestion~~
        +optimize(callback, initial_layout, pinned_keys) Result~OptimizationResult~
    }

    class ScoringSession {
        +Arc~dyn ScoringEngine~ engine
        +Arc~KeycodeRegistry~ registry
        +SearchConfig search_config
    }

    ScoringSession --> Runtime : From
```

`Runtime` can be constructed from a `ScoringSession` via `From` trait.

## 2. Session Builder

The `SessionBuilder` constructs a `ScoringSession` from raw inputs using async asset loading.

```mermaid
classDiagram
    class SessionBuilder~L: AssetLoader~ {
        -loader: &L
        -keyboard: Option~Arc~KeyboardDefinition~~
        -corpus: Option~Arc~Corpus~~
        -rubric: Option~Arc~Rubric~~
        -cost_model: Option~Arc~CostModel~~
        -registry: Option~Arc~KeycodeRegistry~~
        -search_config: Option~SearchConfig~
        -biometrics: Vec~BiometricSample~
        +new(loader) SessionBuilder
        +with_keyboard(name) async LoaderResult~Self~
        +with_keyboard_def(def) Self
        +with_corpus(sources) async LoaderResult~Self~
        +with_corpus_obj(corpus) Self
        +with_cost_matrix(source) async LoaderResult~Self~
        +with_cost_model_obj(model) Self
        +with_keycodes(name) async LoaderResult~Self~
        +with_rubric(rubric) Self
        +with_config(config) Self
        +with_biometrics(samples) Self
        +build() LoaderResult~ScoringSession~
    }
```

### Build Sequence

```mermaid
sequenceDiagram
    participant Client
    participant Builder as SessionBuilder
    participant Loader as AssetLoader
    participant Bio as BiometricProfiler
    participant Factory as EngineFactory

    Client->>Builder: new(loader)
    Client->>Builder: with_keyboard_def(def)
    Client->>Builder: with_corpus(sources)
    Builder->>Loader: load_corpus(sources)
    Loader-->>Builder: Arc<Corpus>
    Client->>Builder: with_cost_matrix(source)
    Builder->>Loader: load<CostModel>(name)
    Loader-->>Builder: Arc<CostModel>
    Client->>Builder: with_keycodes(name)
    Client->>Builder: with_biometrics(samples)
    
    Client->>Builder: build()
    
    alt Has Biometrics
        Builder->>Bio: profile(samples, cost_model)
        Bio-->>Builder: Personalized CostModel
    end
    
    Builder->>Factory: new_generic(keyboard, corpus, rubric, cost_model)
    Factory-->>Builder: Box<dyn ScoringEngine>
    Builder-->>Client: ScoringSession
```

## 3. Biometric Profiler

Transforms raw typing latency samples into personalized cost model adjustments:

```mermaid
classDiagram
    class BiometricProfiler {
        +profile(samples, base_model) CostModel$
    }

    class BiometricSample {
        +bigram: String
        +ms: f64
        +timestamp: u64
    }
```

**Algorithm:**
1. Group samples by bigram
2. Calculate average latency per bigram
3. Compute median latency as normalization baseline
4. Map to `sequence_modifiers` in dynamic rules (requires ≥5 samples per bigram)

## 4. Hardware Detection

CPU topology detection for selecting optimized engine implementations:

```mermaid
classDiagram
    class HardwareProbe {
        +probe() CpuTopology$
    }

    class CpuTopology {
        +vendor: String
        +cache_line_size: u16
        +l1d_size_bytes: usize
        +l2_size_bytes: usize
        +l3_size_bytes: usize
    }

    class IntelEngineConfig {
        +l1d_size_bytes: usize
        +l2_size_bytes: usize
        +l3_size_bytes: usize
        +use_prefetch: bool
    }

    CpuTopology --> IntelEngineConfig : From
```

`HardwareProbe::probe()` uses `raw-cpuid` to detect:
- Vendor (Intel/AMD)
- L1D, L2, L3 cache sizes
- Cache line size

This info feeds into `IntelEngineConfig` for cache-aware blocking in the optimized engine.

## 5. Module Structure

```
keyforge-compute/src/
├── biometrics.rs  # BiometricProfiler
├── builder.rs     # SessionBuilder
├── hardware.rs    # HardwareProbe, CpuTopology
└── lib.rs         # Runtime, exports
```

## 6. Dependencies

| Crate | Purpose |
|-------|---------|
| `keyforge-core` | `ScoringSession`, `AssetLoader`, `ProgressCallback` |
| `keyforge-physics` | `ScoringEngine`, `EngineFactory`, `IntelEngineConfig` |
| `keyforge-model` | Domain types |
| `keyforge-protocol` | `BiometricSample` DTO |
| `raw-cpuid` | CPU feature detection |
