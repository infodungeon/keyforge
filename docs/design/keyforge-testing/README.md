# Design: KeyForge Testing

**Responsibility:** Hermetic Test Harness & Fixtures.
**Tier:** 3 (The Tooling)

## 1. The Hermetic Workspace

The `HermeticWorkspace` struct provides an isolated, file-based environment for integration tests. It creates a temporary directory and populates it with "Golden Data" (Corpus, Keyboards, Weights).

```rust
use keyforge_testing::HermeticWorkspace;

#[test]
fn test_full_lifecycle() {
    let ws = HermeticWorkspace::new()
        .with_keyboard("ansi", default_ansi())
        .with_corpus("english", "the", 100);
    
    // Run test against ws.data_root
}
```

## 2. Golden Data

This crate contains the canonical "Truth" for testing:
* **Standard Keyboards:** ANSI, ISO, ErgoDox.
* **Standard Corpora:** English, Code.
* **Standard Rubrics:** Default weights.
