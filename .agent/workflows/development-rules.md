# KeyForge Development Rules: The Law and Grace

## 1. Semantic Sovereignty
- **Truth Oracle**: `cargo clippy` and the Rust compiler are the only sources of truth. `grep` and `sed` are for discovery only.
- **Structural Integrity**: Treat the codebase as an Abstract Syntax Tree (AST). All batch modifications must be verified for syntactic correctness (balanced braces, valid struct order) before being committed to disk.

## 2. Leverage through Unification
- **The Oracle of Grace**: `libs/keyforge-testing-macros::kf_test` is the sole allowed location for `#[allow]` attributes in tests.
- **Master Pivots**: Move logic toward the "Source of Truth" (core model). Infrastructure must be a projection of the core, not a manual mapping.

## 3. The 100x Gate
- **just 100x**: Mandatory command before finishing any turn. Runs structural audits and strict linting.
- **Zero Rework**: Every move must be right the first time through mental simulation and single-slice proofing.