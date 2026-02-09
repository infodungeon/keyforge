**ETS-100x Standard Ticket**

### [ARCH-SEC-001] Critical: Systemic Path Traversal (CWE-22) Remediation

**Context**
The `narsil` security scanner identified 102 Path Traversal vulnerabilities across the codebase, specifically in `keyforge-persistence`, `keyforge-infra`, and `keyforge-cli`.

**Evidence**
Example trace: `libs/keyforge-persistence/src/repo/user_repo.rs:62` -> Direct use of `fs::read_to_string` with unvalidated path components.
Narsil Findings: 102 occurrences of CWE-22.

**Violation**
ARCH-005 (Hexagonal Purity / Safe Boundaries) and ADR-006 (Universal Validation).

**Required Deliverable**
1. **Model Implementation:** Implement `SafePath` newtype in `keyforge-model` as specified in **ADR-027**.
2. **Persistence Refactor:** Update `libs/keyforge-persistence` and `libs/keyforge-infra` to use `SafePath` for all file-based repository operations.
3. **CLI Refactor:** Update `apps/keyforge-cli` commands (e.g., `auth`, `profile`) to validate input paths via `SafePath`.
4. **Cleanup:** Replace ad-hoc `normalize_path` calls with `SafePath` usage.

**Mandatory Verification Gates (Justfile)**
- `just audit`: Zero Path Traversal findings in `narsil` scan for core libraries.
- `just test-all`: All persistence tests must pass with the new path type.

**Scope Boundary**
`keyforge-model`, `keyforge-persistence`, `keyforge-infra`, `keyforge-cli`.

**Labels:** `architecture`, `security`, `remediation`, `P0`