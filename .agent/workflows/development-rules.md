# KeyForge Development Rules: The Law and Grace

## 1. Semantic Sovereignty
- **Truth Oracle**: `cargo clippy` and the Rust compiler are the only sources of truth. `grep` and `sed` are for discovery only.
- **SEARCH-001: Targeted Search**: Root-level searches (`./`) are strictly forbidden. All exploration must be targeted to subdirectories (`libs/`, `apps/`, `tests/`) and use specific file extensions to prevent context overflow.
- **SEARCH-002: Regression Isolation**: Any search tool (`grep`, `rg`, `search_file_content`) MUST explicitly exclude `proptest-regressions` paths. Large files with single-line lengths > 1000 characters are "Exclusion Zones".
- **SEARCH-003: Match Capping**: When using shell-based search, always pipe through `cut -c 1-500` or use `grep -o` to prevent a single massive line from overwhelming the context window.
- **Structural Integrity**: Treat the codebase as an Abstract Syntax Tree (AST). All batch modifications must be verified for syntactic correctness (balanced braces, valid struct order) before being committed to disk.

## 2. Leverage through Unification
- **The Oracle of Grace**: `libs/keyforge-testing-macros::kf_test` is the sole allowed location for `#[allow]` attributes in tests.
- **Master Pivots**: Move logic toward the "Source of Truth" (core model). Infrastructure must be a projection of the core, not a manual mapping.

## 3. The 100x Gate
- **just 100x**: Mandatory command before finishing any turn. Runs structural audits and strict linting.
- **Zero Rework**: Every move must be right the first time through mental simulation and single-slice proofing.