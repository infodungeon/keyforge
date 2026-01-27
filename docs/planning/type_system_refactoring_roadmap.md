# KeyForge Type System Refactoring Roadmap

**Created:** 2026-01-27  
**Author:** Architecture Analysis Session  
**Status:** Planning

---

## 🎯 Executive Summary

KeyForge's type system requires systematic refactoring to achieve:

1. **Idiomatic Rust** - Flat structs, minimal nesting, zero-cost abstractions
2. **Architectural Correctness** - Proper layer separation (Protocol vs Model)
3. **Consistency** - Uniform naming (DTO suffix), clear Entity vs Value Object distinction
4. **Maintainability** - Eliminate compound entities, reduce primitive obsession

**Estimated Effort:** ~40-60 hours across 6 phases  
**Breaking Changes:** YES (major version bump required)  
**Risk Level:** HIGH (touches core domain model)

---

## 📊 Current State Assessment

### Issues Identified

| Category | Count | Severity | Examples |
|----------|-------|----------|----------|
| Layer Violations | 2 | 🔴 Critical | `RawCostModel` in model layer, `KeyboardGeometryDto` duplicated |
| Compound Entities | 5 | 🔴 Critical | `AnalysisReport`, `Keyboard`, `Corpus`, `KeyboardDefinition`, `EngineContext` duplication |
| Missing DTOs | 16 | 🟡 High | `Corpus`, `Rubric`, `Keyboard`, `GeometryData`, etc. |
| Nested Metadata | 4 | 🟢 Medium | `KeyboardMeta`, `CostModelMeta`, `ProjectMeta`, `CorpusMetadata` |
| Missing Entities | 7 | 🟡 High | `UserProfile`, `BiometricProfile`, `ScoringContext`, etc. |
| Primitive Obsession | 8 | 🟢 Medium | `String` for IDs, `Vec<(u16,u16,u32)>` for frequencies |

**Total Issues:** 42  
**Technical Debt Impact:** HIGH - affects all 13 crates

---

## 🗺️ Phased Refactoring Plan

### Phase 0: Foundation (Week 1)

**Goal:** Establish contracts and freeze breaking changes

- [ ] Create architectural decision records (ADRs)
- [ ] Document Entity vs DTO vs Value Object patterns
- [ ] Set up deprecation strategy
- [ ] Create migration guide template

**Output:** `docs/architecture/adr/`

---

### Phase 1: Critical Layer Violations (Week 2-3)

**Goal:** Fix immediate architectural violations

**Priority:** 🔴 CRITICAL  
**Breaking Changes:** YES  
**Issues:** 3

#### Issues

1. **Migrate `RawCostModel` → `CostModelDto`**
   - Move from `keyforge-model` to `keyforge-protocol`
   - Rename to follow DTO convention
   - Update all 17 references across crates
   - **Dependencies:** None
   - **Complexity:** HIGH (cross-crate)

2. **Deduplicate `KeyboardGeometryDto`**
   - Consolidate definitions from `assets.rs` and `config.rs`
   - Ensure single source of truth in `protocol/assets.rs`
   - **Dependencies:** None
   - **Complexity:** MEDIUM

3. **Extract Missing Component DTOs**
   - Create `ModelDefinitionDto`
   - Create `DynamicRulesDto`
   - Create `FingerReachDto`
   - Move to `keyforge-protocol`
   - **Dependencies:** Issue #1
   - **Complexity:** MEDIUM

**Deliverables:**

- [ ] `keyforge-protocol/src/assets.rs` contains `CostModelDto`
- [ ] All `*Dto` types have single definition
- [ ] `keyforge-model` imports from protocol, not vice versa
- [ ] Zero Clippy warnings

---

### Phase 2: Split Compound Entities (Week 4-5)

**Goal:** Decompose God Objects following SRP

**Priority:** 🔴 CRITICAL  
**Breaking Changes:** YES  
**Issues:** 5

#### Issue #4: Split `AnalysisReport`

**Current:** 16 fields, 4 concerns mixed

**Refactor to:**

```rust
pub struct AnalysisReport {
    pub summary: ScoreSummary,
    pub breakdown: MetricBreakdown,
    pub travel: TravelStatistics,
    pub visualizations: Heatmaps,
}

pub struct ScoreSummary {
    pub total_score: f32,
    pub metrics: MetricSet,
}

pub struct MetricBreakdown {
    pub violations: HashMap<MetricId, Vec<MetricViolation>>,
}

pub struct TravelStatistics {
    pub distance: f32,
    pub travel_per_key: f32,
    pub hand_balance: f32,
}

pub struct Heatmaps {
    pub usage: Vec<f32>,
    pub effort: Vec<f32>,
}
```

**Impact:** `keyforge-physics`, `keyforge-ui`, `keyforge-hive`  
**Complexity:** HIGH  
**Acceptance Criteria:**

- [ ] Each sub-type has single responsibility
- [ ] No duplicate data (remove `top_sfbs`, `top_scissors` - derive from violations)
- [ ] DTOs created for each type
- [ ] All tests pass

---

#### Issue #5: Split `Keyboard`

**Current:** Mixes domain + performance cache

**Refactor to:**

```rust
// Domain entity
pub struct Keyboard {
    pub metadata: KeyboardMetadata,
    pub geometry: KeyboardGeometry,
}

pub struct KeyboardMetadata {
    pub kb_type: String,
    pub home_row: RowIndex,
}

// Performance layer (separate from domain)
pub struct SpatialIndex {
    finger_origins: Vec<Vec<(f32, f32)>>,
    spatial_cache: Vec<(f32, f32)>,
}

impl SpatialIndex {
    pub fn build_from(keyboard: &Keyboard) -> Self { ... }
}
```

**Impact:** `keyforge-model`, `keyforge-physics`  
**Complexity:** HIGH  
**Acceptance Criteria:**

- [ ] Domain layer has no cached computations
- [ ] `SpatialIndex` in separate module or `keyforge-physics`
- [ ] Performance benchmarks show no regression

---

#### Issue #6: Split `Corpus` + Extract Service

**Current:** 5 data structures + merge logic in entity

**Refactor to:**

```rust
pub struct Corpus {
    pub metadata: CorpusMetadata,
    pub frequencies: FrequencyTables,
}

pub struct FrequencyTables {
    pub char_freqs: Arc<[u64]>,
    pub bigrams: BigramFrequencyTable,
    pub trigrams: TrigramFrequencyTable,
    pub words: Arc<[(String, u32)]>,
}

pub struct CorpusMerger;  // Service, not entity method

impl CorpusMerger {
    pub fn merge(base: Corpus, other: &Corpus, weight: f32) -> Corpus { ... }
}
```

**Impact:** `keyforge-model`  
**Complexity:** MEDIUM  
**Acceptance Criteria:**

- [ ] Merge logic in service, not entity
- [ ] Value objects for bigram/trigram tables
- [ ] DTOs created

---

#### Issue #7: Split `KeyboardDefinition`

**Current:** Violates aggregate boundary with `layouts` field

**Refactor to:**

```rust
pub struct KeyboardDefinition {
    pub metadata: KeyboardMetadata,
    pub geometry: KeyboardGeometry,
    // Remove layouts field
}

pub struct LayoutCatalog {
    keyboard_id: KeyboardId,
    layouts: HashMap<LayoutName, LayoutData>,
}
```

**Impact:** `keyforge-model`, all deserializers  
**Complexity:** MEDIUM  
**Acceptance Criteria:**

- [ ] Keyboard definition only describes physical device
- [ ] Layout catalog is separate entity
- [ ] Migration path for existing JSON files

---

#### Issue #8: Unify `EngineCompilationContext` + `EngineContext`

**Current:** Two types for same concept (uncompiled vs compiled)

**Refactor to:**

```rust
pub struct ScoringContext<S: ScoringContextState> {
    inner: S::Data,
    _marker: PhantomData<S>,
}

pub struct Uncompiled;
pub struct Compiled;

impl ScoringContext<Uncompiled> {
    pub fn compile(self) -> Result<ScoringContext<Compiled>, PhysicsError> { ... }
}
```

**Impact:** `keyforge-physics`, `keyforge-model`  
**Complexity:** HIGH (typestate pattern, cross-crate)  
**Acceptance Criteria:**

- [ ] Single entity with typestate
- [ ] Compilation is explicit in type system
- [ ] No duplication of context fields

---

### Phase 3: Flatten Nested Metadata (Week 6)

**Goal:** Remove unnecessary nesting for idiomatic Rust

**Priority:** 🟢 MEDIUM  
**Breaking Changes:** YES (but minor)  
**Issues:** 4

#### Issue #9-12: Flatten Metadata Structs

**For each:** `KeyboardMeta`, `CostModelMeta`, `ProjectMeta`, `CorpusMetadata`

**Approach:**

1. Check if JSON has `"meta": {...}` object
   - **If YES:** Keep nesting, add `#[serde(flatten)]` if needed
   - **If NO:** Inline fields to parent struct
2. Update all field accesses (`obj.meta.name` → `obj.name`)
3. Update DTOs to match

**Per-struct decision:**

| Struct | JSON Has "meta"? | Decision | Complexity |
|--------|------------------|----------|------------|
| `KeyboardMeta` | TBD (check files) | Flatten if no | LOW |
| `CostModelMeta` | TBD (check files) | Flatten if no | LOW |
| `ProjectMeta` | TBD (check files) | Flatten if no | LOW |
| `CorpusMetadata` | N/A (1 field!) | ALWAYS flatten | LOW |

**Acceptance Criteria (per struct):**

- [ ] Fields inline if JSON is flat
- [ ] All references updated
- [ ] Serde tests pass

---

### Phase 4: Create Missing DTOs (Week 7-8)

**Goal:** Complete protocol layer coverage

**Priority:** 🟡 HIGH  
**Breaking Changes:** NO (additive)  
**Issues:** 10

#### Missing DTOs to Create

| Domain Type | DTO to Create | Location | Complexity |
|-------------|---------------|----------|------------|
| `Corpus` | `CorpusDto` | `protocol/assets.rs` | MEDIUM |
| `Rubric` | `RubricDto` | `protocol/config.rs` | MEDIUM |
| `Keyboard` | `KeyboardDto` | `protocol/assets.rs` | LOW |
| `GeometryData` | `GeometryDataDto` | `protocol/assets.rs` | LOW |
| `CorpusData` | `CorpusDataDto` | `protocol/assets.rs` | LOW |
| `OptimizationResult` | `OptimizationResultDto` | `protocol/types.rs` | LOW |
| `FrequencyTables` | `FrequencyTablesDto` | `protocol/assets.rs` | MEDIUM |
| `ScoreSummary` | `ScoreSummaryDto` | `protocol/types.rs` | LOW |
| `MetricBreakdown` | `MetricBreakdownDto` | `protocol/types.rs` | LOW |
| `Heatmaps` | `HeatmapsDto` | `protocol/types.rs` | LOW |

**Template per DTO:**

```rust
// keyforge-protocol/src/assets.rs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CorpusDto {
    pub metadata: CorpusMetadataDto,
    pub char_freqs: Vec<u64>,
    pub bigrams: Vec<(u16, u16, u32)>,
    pub trigrams: Vec<(u16, u16, u16, u32)>,
    pub words: Vec<(String, u32)>,
}

// keyforge-model/src/corpus.rs
impl From<CorpusDto> for Corpus {
    fn from(dto: CorpusDto) -> Self {
        Self {
            metadata: dto.metadata.into(),
            char_freqs: Arc::from(dto.char_freqs),
            bigrams: Arc::from(dto.bigrams),
            trigrams: Arc::from(dto.trigrams),
            words: Arc::from(dto.words),
        }
    }
}

impl From<&Corpus> for CorpusDto {
    fn from(corpus: &Corpus) -> Self {
        Self {
            metadata: (&corpus.metadata).into(),
            char_freqs: corpus.char_freqs.to_vec(),
            bigrams: corpus.bigrams.to_vec(),
            trigrams: corpus.trigrams.to_vec(),
            words: corpus.words.to_vec(),
        }
    }
}
```

**Acceptance Criteria:**

- [ ] All boundary-crossing types have DTOs
- [ ] DTOs in `keyforge-protocol` only
- [ ] `From`/`TryFrom` impls for conversion
- [ ] Serde round-trip tests

---

### Phase 5: Create Missing Domain Entities (Week 9-10)

**Goal:** Fill architectural gaps with new entities

**Priority:** 🟡 HIGH  
**Breaking Changes:** NO (additive)  
**Issues:** 7

#### Issue #23: Create `UserProfile` Aggregate

```rust
pub struct UserProfile {
    pub id: UserId,
    pub username: String,
    pub preferences: UserPreferences,
    pub biometrics: BiometricProfile,
    pub submission_history: Vec<SubmissionId>,
    pub created_at: DateTime<Utc>,
}

pub struct UserPreferences {
    pub default_keyboard: Option<KeyboardId>,
    pub default_corpus: Option<CorpusId>,
    pub theme: UiTheme,
}
```

**Impact:** `keyforge-persistence`, `keyforge-hive`  
**Complexity:** MEDIUM

---

#### Issue #24: Create `BiometricProfile`

```rust
pub struct BiometricProfile {
    user_id: UserId,
    samples: Vec<BiometricSample>,
    aggregated_timings: AggregatedTimings,
}

pub struct AggregatedTimings {
    pub mean_keydown_time: DurationMs,
    pub std_dev: DurationMs,
    pub percentile_95: DurationMs,
}
```

**Impact:** `keyforge-model`, `keyforge-persistence`  
**Complexity:** MEDIUM

---

#### Issue #25: Create `LayoutSubmission`

```rust
pub struct LayoutSubmission {
    pub id: SubmissionId,
    pub submitter: UserId,
    pub layout: NamedLayout,
    pub score: Score,
    pub submitted_at: DateTime<Utc>,
    pub status: SubmissionStatus,
}

pub enum SubmissionStatus {
    Pending,
    Approved,
    Rejected { reason: String },
}
```

**Impact:** `keyforge-hive`  
**Complexity:** LOW

---

#### Issue #26: Create `AnalysisSession`

```rust
pub struct AnalysisSession {
    session_id: Ulid,
    config: AnalysisConfig,
    layout: Layout,
    partial_results: Option<PartialAnalysisReport>,
    started_at: DateTime<Utc>,
}
```

**Impact:** `keyforge-physics`, `keyforge-ui`  
**Complexity:** MEDIUM

---

#### Issue #27: Create `JobExecution`

```rust
pub struct JobExecution {
    job_id: JobIdentifier,
    config: JobConfig,
    current_state: ExecutionState,
    wal_entries: Vec<WalEntry>,
    partial_result: Option<OptimizationResult>,
}
```

**Impact:** `keyforge-hive`  
**Complexity:** HIGH

---

#### Issue #28: Create `KeyboardInventory`

```rust
pub struct KeyboardInventory {
    available_keyboards: HashMap<KeyboardId, KeyboardSummary>,
}

pub struct KeyboardSummary {
    id: KeyboardId,
    name: String,
    key_count: usize,
    kb_type: String,
}
```

**Impact:** `keyforge-persistence`, `keyforge-ui`  
**Complexity:** LOW

---

#### Issue #29: Create `LayoutCatalog`

```rust
pub struct LayoutCatalog {
    keyboard_id: KeyboardId,
    layouts: HashMap<LayoutName, LayoutEntry>,
}

pub struct LayoutEntry {
    name: LayoutName,
    data: LayoutData,
    created_at: DateTime<Utc>,
    tags: Vec<String>,
}
```

**Impact:** `keyforge-model`  
**Complexity:** MEDIUM

---

### Phase 6: Eliminate Primitive Obsession (Week 11)

**Goal:** Wrap primitives in value objects

**Priority:** 🟢 MEDIUM  
**Breaking Changes:** YES (but localized)  
**Issues:** 8

#### Value Objects to Create

```rust
// IDs
pub struct CorpusId(String);
pub struct KeyboardId(String);  // Unify String/i32 usage
pub struct LayoutId(Ulid);
pub struct LayoutName(String);
pub struct SubmissionId(Ulid);
pub struct UserId(Ulid);

// Measurements
pub struct Percentage(f32);  // 0.0-100.0, validated
pub struct Ratio(f32);       // 0.0-1.0, validated

// Sequences
pub struct BigramFrequencyTable(Arc<[(u16, u16, u32)]>);
pub struct TrigramFrequencyTable(Arc<[(u16, u16, u16, u32)]>);
pub struct SequenceModifiers(HashMap<(u16, u16), Score>);

// Collections
pub struct LayoutCatalogData(HashMap<LayoutName, LayoutData>);
```

**Acceptance Criteria (per value object):**

- [ ] Validation in constructor
- [ ] Implements `Display`, `Debug`, `Serialize`, `Deserialize`
- [ ] Zero-cost wrapper (newtype pattern)
- [ ] Used consistently across codebase

---

## 📋 GitHub Issue Template

### Template for Each Issue

```markdown
## 🎯 Objective
[Clear statement of what needs to change]

## 📊 Current State
```rust
// Current problematic code
```

## ✅ Desired State

```rust
// Target architecture
```

## 🔧 Implementation Steps

1. [ ] Step 1
2. [ ] Step 2
3. [ ] Step 3

## 🧪 Acceptance Criteria

- [ ] All tests pass
- [ ] Zero Clippy warnings
- [ ] Benchmark shows no regression (if applicable)
- [ ] Documentation updated

## 📦 Files to Modify

- `path/to/file1.rs`
- `path/to/file2.rs`

## 🔗 Dependencies

- Blocked by: #123
- Blocks: #456

## 💡 Migration Notes

[How existing code/data migrates]

## ⚠️ Breaking Changes

YES/NO - [explanation]

## 📏 Complexity

LOW/MEDIUM/HIGH

## ⏱️ Estimated Effort

X hours

```

---

## 🚦 Risk Management

### High-Risk Changes

| Issue | Risk | Mitigation |
|-------|------|------------|
| Phase 1: `RawCostModel` migration | Cross-crate breakage | Feature flag, dual import period |
| Phase 2: Split entities | Massive refactor | Split into sub-PRs, comprehensive tests |
| Phase 3: Flatten metadata | Serde breakage | JSON compatibility tests |

### Rollback Strategy

1. Each phase is a feature branch
2. Feature flags for dual implementation period
3. Deprecation warnings before removal
4. Semantic versioning: `v2.0.0` for breaking changes

---

## 📈 Success Metrics

| Metric | Before | Target | How to Measure |
|--------|--------|--------|----------------|
| DTO Coverage | 65% | 95% | Count types crossing boundaries |
| Layer Violations | 2 | 0 | `ast-grep` audit |
| Compound Entities | 5 | 0 | SRP audit |
| Nested Metadata | 4 | 0-1 | Struct depth analysis |
| Primitive IDs | 8 | 0 | Type audit |
| Clippy Warnings | Current | 0 | `cargo clippy --all-features` |

---

## 🗓️ Timeline

```

Week 1:  Phase 0 - Foundation
Week 2-3: Phase 1 - Layer Violations (🔴 Critical)
Week 4-5: Phase 2 - Split Entities (🔴 Critical)
Week 6:   Phase 3 - Flatten Metadata (🟢 Medium)
Week 7-8: Phase 4 - Missing DTOs (🟡 High)
Week 9-10: Phase 5 - Missing Entities (🟡 High)
Week 11:  Phase 6 - Value Objects (🟢 Medium)
Week 12:  Documentation, benchmarks, polish

```

**Total Duration:** 12 weeks (3 months)  
**Parallel Work Possible:** Phases 3, 4, 5, 6 can be parallelized after Phase 2

---

## 🎓 Learning Resources

For team members implementing these changes:

- **ADR-001:** Entity vs DTO vs Value Object
- **ADR-002:** When to Nest vs Flatten
- **ADR-003:** Newtype Pattern for Primitives
- **ADR-004:** Typestate Pattern for Lifecycle
- **Rust Book:** Ch. 19 - Advanced Types
- **DDD in Rust:** [Link to resources]

---

## 🔍 Continuous Audit

After completion, add to CI:

```bash
# Check for new layer violations
just audit-layers

# Check DTO coverage
just audit-dtos

# Check for primitive obsession
just audit-primitives
```

**Goal:** Prevent regression via mechanical verification.
