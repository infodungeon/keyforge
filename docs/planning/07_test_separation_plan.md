# Test Separation Plan: Unit vs Integration Tests

**Version:** 1.0  
**Date:** 2026-01-20  
**Status:** Draft - Pending Review

## 1. Objective

Separate integration tests from unit tests across all `libs/` crates to align with ADR-015 (Data Decoupling and Testing Strategy):

- **Unit Tests (`src/`)**: Rigorous, exhaustive verification of logic and math
- **Integration Tests (`tests/`)**: Contract/wiring verification only
- **Zero Duplication**: Unit logic MUST NOT be re-verified in the integration layer

## 2. Current State Analysis

| Metric | Count |
|--------|-------|
| Inline `#[cfg(test)]` modules | 91 |
| Integration test files (`tests/*.rs`) | 9 |
| Crates with `tests/` directory | 10 |
| Crates missing `tests/` directory | 4 (`keyforge-export`, `keyforge-security`, `keyforge-wasm`, `keyforge-testing`) |

### Crates by Criticality Tier

| Tier | Crates | Testing Requirement |
|------|--------|---------------------|
| **Tier 1 (Nucleus)** | `keyforge-physics`, `keyforge-evolution` | 95%+ Branch Coverage + Property Testing |
| **Tier 2 (Contract)** | `keyforge-protocol`, `keyforge-model` | 100% Validation Coverage |
| **Tier 3 (Shell)** | `keyforge-infra`, `keyforge-persistence`, `keyforge-adapter`, `keyforge-compute`, `keyforge-core`, `keyforge-runner` | Error Path Verification |
| **Utility** | `keyforge-export`, `keyforge-security`, `keyforge-wasm`, `keyforge-testing` | Standard Coverage |

## 3. Classification Criteria

### Unit Test (stays in `src/`)
- Tests a single function or struct in isolation
- Uses no external resources (filesystem, network, database)
- Mocks/stubs all dependencies
- Tests pure logic, math, algorithms, validation
- Fast execution (< 10ms per test)

### Integration Test (moves to `tests/`)
- Tests interaction between multiple modules/crates
- Uses `tempfile`, `tokio::test`, or real IO
- Verifies wiring/contracts between components
- Uses fixtures from external files
- Tests public API surface across crate boundaries

## 4. Identified Migration Candidates

Based on code review, the following inline tests appear to be integration tests:

### 4.1 `keyforge-infra` (High Priority)
| File | Test Module | Reason |
|------|-------------|--------|
| `src/asset/fs_provider.rs` | `tests` | Uses `tempfile`, filesystem IO, async |
| `src/asset/valkey_provider.rs` | `tests` | Network mocking, async coordination |
| `src/asset/caching_provider.rs` | `tests` | Combines filesystem + caching layers |
| `src/fs/io.rs` | `tests` | File read/write operations |
| `src/fs/listing.rs` | `tests` | Directory traversal |
| `src/fs/lock.rs` | `tests` | File locking mechanics |
| `src/net/network.rs` | `tests` | HTTP client behavior |
| `src/net/sync.rs` | `tests` | Sync protocol verification |

### 4.2 `keyforge-persistence` (High Priority)
| File | Test Module | Reason |
|------|-------------|--------|
| `src/repo/user_repo.rs` | `tests` | Repository pattern wiring |
| `src/store/autosave.rs` | `tests` | Filesystem + timer interaction |
| `src/compiler.rs` | `tests` | Multi-stage compilation pipeline |
| `src/project.rs` | `tests` | Project lifecycle management |

### 4.3 `keyforge-physics` (Medium Priority)
| File | Test Module | Reason |
|------|-------------|--------|
| `src/verify.rs` | `tests` | Oracle vs Engine parity (property tests) |
| `src/engines/generic.rs` | `tests` | Engine factory integration |
| `src/engines/exact.rs` | `tests` | Cross-engine comparison |
| `src/engines/intel_comet_lake.rs` | `tests` | Hardware-specific integration |

### 4.4 `keyforge-evolution` (Medium Priority)
| File | Test Module | Reason |
|------|-------------|--------|
| `src/supervisor/annealing.rs` | `tests` | Full annealing loop with engine |
| `src/supervisor/optimizer.rs` | `tests` | Optimizer orchestration |
| `src/supervisor/state.rs` | `tests` | State machine transitions |

### 4.5 `keyforge-model` (Low Priority - mostly unit tests)
| File | Test Module | Reason |
|------|-------------|--------|
| `src/geometry/kle.rs` | `tests` | KLE parser (could use fixture files) |
| `src/parsing.rs` | `tests` | Layout parsing (could use fixtures) |

### 4.6 Other Crates (Low Priority)
| Crate | Files | Notes |
|-------|-------|-------|
| `keyforge-compute` | `src/hardware.rs`, `src/builder.rs` | Builder pattern wiring |
| `keyforge-core` | `src/session.rs` | Session lifecycle |
| `keyforge-runner` | `src/lib.rs` | Runner orchestration |
| `keyforge-wasm` | `src/lib.rs`, `src/loader.rs` | WASM boundary tests |
| `keyforge-export` | `src/zmk.rs`, `src/via.rs`, `src/qmk.rs` | Export format generation |

## 5. Task Breakdown

### Phase 1: Infrastructure Setup (Estimated: 2 hours)
- [ ] **T1.1**: Create `tests/` directories for crates missing them
  - `keyforge-export`, `keyforge-security`, `keyforge-wasm`, `keyforge-testing`
- [ ] **T1.2**: Create `tests/fixtures/` directories where needed
- [ ] **T1.3**: Establish shared test utilities in `keyforge-testing` crate
- [ ] **T1.4**: Document test file naming convention in AGENTS.md

### Phase 2: Tier 3 Crates - Shell Layer (Estimated: 4 hours)
- [ ] **T2.1**: `keyforge-infra` - Migrate 8 integration test modules
  - Extract async/IO tests from `fs_provider.rs`, `valkey_provider.rs`, etc.
  - Consolidate into `tests/provider_integration.rs`, `tests/fs_integration.rs`, `tests/net_integration.rs`
- [ ] **T2.2**: `keyforge-persistence` - Migrate 4 integration test modules
  - Extract to `tests/repository_integration.rs` (expand existing)
- [ ] **T2.3**: `keyforge-adapter` - Review 3 conversion modules
  - Migrate cross-module tests to `tests/translation_integration.rs`
- [ ] **T2.4**: `keyforge-compute` - Migrate builder/hardware tests
- [ ] **T2.5**: `keyforge-core` - Migrate session orchestration tests
- [ ] **T2.6**: `keyforge-runner` - Migrate runner lifecycle tests

### Phase 3: Tier 1 Crates - Nucleus (Estimated: 3 hours)
- [ ] **T3.1**: `keyforge-physics` - Migrate engine integration tests
  - Keep pure math tests (geometry, cost calculations) as unit tests
  - Move engine factory, oracle parity, cross-engine tests to `tests/`
  - Expand `tests/fixtures/` with golden data files
- [ ] **T3.2**: `keyforge-evolution` - Migrate supervisor tests
  - Keep mutation operator logic as unit tests
  - Move full annealing loop, state machine tests to `tests/`

### Phase 4: Tier 2 Crates - Contract Layer (Estimated: 2 hours)
- [ ] **T4.1**: `keyforge-protocol` - Review and classify
  - Serialization round-trip tests → may stay as unit tests
  - Validation integration → move if testing multiple validators together
- [ ] **T4.2**: `keyforge-model` - Review and classify
  - Pure struct tests → stay as unit tests
  - Parser tests with fixtures → move to `tests/` with external fixture files

### Phase 5: Utility Crates (Estimated: 1 hour)
- [ ] **T5.1**: `keyforge-export` - Migrate format generation tests
- [ ] **T5.2**: `keyforge-security` - Review (likely stays as unit tests)
- [ ] **T5.3**: `keyforge-wasm` - Migrate boundary tests
- [ ] **T5.4**: `keyforge-testing` - Meta-review of test utilities

### Phase 6: Cleanup & Documentation (Estimated: 2 hours)
- [ ] **T6.1**: Remove duplicated test logic between unit and integration
- [ ] **T6.2**: Add Intent/Expected Result documentation to all integration tests
- [ ] **T6.3**: Update `16_TESTING_STANDARDS.md` with concrete examples
- [ ] **T6.4**: Run full test suite and verify coverage maintained
- [ ] **T6.5**: Update AGENTS.md with integration test running commands

## 6. Migration Template

When moving a test module from `src/foo.rs` to `tests/foo_integration.rs`:

```rust
// tests/foo_integration.rs

//! Integration tests for the Foo subsystem.
//!
//! ## Scope
//! - Contract verification between Foo and Bar
//! - End-to-end wiring validation
//!
//! ## NOT Tested Here (see unit tests in src/)
//! - Pure logic validation
//! - Individual function behavior

use keyforge_foo::{Foo, FooConfig};
use keyforge_testing::fixtures::load_fixture;

/// Intent: Verify Foo correctly wires to Bar when configured with defaults.
/// Expected: Foo.process() returns Ok with valid output matching fixture.
#[test]
fn test_foo_bar_wiring() {
    // Arrange
    let fixture = load_fixture("foo_input.json");
    
    // Act
    let result = Foo::new(FooConfig::default()).process(&fixture);
    
    // Assert
    assert!(result.is_ok());
}
```

## 7. Success Criteria

- [ ] All integration tests reside in `tests/` directories
- [ ] No filesystem/network/async operations in `src/` test modules
- [ ] Zero test logic duplication between unit and integration layers
- [ ] All integration tests have Intent/Expected Result documentation
- [ ] Coverage remains at or above current levels
- [ ] `cargo test -p <crate> --lib` runs only unit tests
- [ ] `cargo test -p <crate> --test '*'` runs only integration tests

## 8. Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Coverage drop during migration | Run `just cover <crate>` before and after each phase |
| Breaking existing CI | Incremental migration, verify each crate passes before continuing |
| Shared test utilities not available | Ensure `keyforge-testing` is a dev-dependency in all crates |
| Fixture files missing | Create fixtures alongside test migration |

## 9. Open Questions

1. Should property tests (proptest) remain in `src/` or move to `tests/`?
   - **Recommendation**: Keep in `src/` if testing single-function properties; move to `tests/` if testing multi-module integration properties

2. Should we introduce a `#[integration_test]` attribute or rely on directory convention?
   - **Recommendation**: Directory convention is sufficient and standard for Rust

3. How to handle tests that need both unit and integration variants?
   - **Recommendation**: Parameterize in integration tests; don't duplicate assertions

---

**Next Steps**: Review and approve this plan before execution begins.
