# ISSUE-002: Test Grace Proliferation

**Goal**: Eliminate manual repetition of `#[allow]` in test files by hardening the `kf_test` macro.

**Status**: OPEN
**Phase**: 1. Compliance Audit

## Findings
- Manual repetition of `#[allow]` in 89+ files.
- Violation of ARCH-006 (Structural Oracle).

## Plan (100x Filter)
- **Lever 3: Macros**: Update `kf_test` macro to automatically include the necessary `#[allow]` attributes for test targets.
