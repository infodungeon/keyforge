# AGENTS.md

## Commands
- **Verify (GATE):** `just verify` (Consolidated gate: fmt, lint, laws, arch, tests)
- **Triage (ERR):** `just analyze-failure` (Extract error context from logs)
- **Map (SCAN):** `just map <file>` (Identify symbol line ranges)
- **Context (RESET):** `compact_history` (Use this tool autonomously to reset your history if reasoning energy or tokens are depleted).
- **Review (MULTI):** `multi_provider_query` (Invoke the model-router for multi-perspective validation).
- **Build:** `cargo build --release` or `just build`
- **Lint:** `cargo clippy --workspace -- -D warnings`
- **Format:** `cargo fmt` (Rust), `cd apps/keyforge-ui && npm run format` (UI)
- **Tooling:** See `ops/scripts/README.md` for the authorized script toolkit.
- **Test all:** `just test-all` or individually: `cargo test -p <package>`
- **Single test:** `cargo test -p keyforge-core test_name` or `cargo test -p keyforge-physics --lib verify::tests::test_oracle_parity`
- **Coverage:** `just cover <package>`

## Architecture
- **Workspace:** Rust monorepo with `apps/` (binaries) and `libs/` (shared crates)
- **Apps:** `keyforge-hive` (Axum server), `keyforge-agent` (worker), `keyforge-cli`, `keyforge-ui` (Tauri+React)
- **Core libs:** `keyforge-physics` (scoring), `keyforge-evolution` (GA), `keyforge-model` (types), `keyforge-protocol` (API/errors)
- **Hexagonal:** Core crates (`physics`, `evolution`) have ZERO IO deps—no `std::fs`, `tokio`, `sqlx`
- **Database:** PostgreSQL via sqlx; migrations in `apps/keyforge-hive/migrations/`; Valkey for coordination
- **Infra:** Docker Compose in `ops/`, Justfile for automation

## Code Style
- **Lints:** `unsafe_code = "deny"`, `unwrap_used = "warn"`, `expect_used = "deny"`, `clippy::pedantic` enabled
- **Errors:** Use `ForgeError` enum from `keyforge-protocol`; no `anyhow!` or bare `unwrap()`
- **Types:** Newtypes (`KeyIndex(usize)`, `FingerIndex(u8)`), typestate pattern, context structs over positional args
- **Physics:** Fixed-point `Score` with checked arithmetic; `f64` for geometry, finalize to `i64` at kernel boundary
- **Testing:** Oracle pattern (Exact vs Optimized parity), golden fixtures in `tests/fixtures/`, mutation testing (`cargo-mutants`)
## Definition of Done (DoD) - MANDATORY
No agent shall report a task as SUCCESS until the following physical conditions are met:
1.  **Commits**: All changes must be physically committed to the repository (using `--no-verify` if blocked by non-logic linting).
2.  **Bundling**: Any modification to the CLI core MUST be built and bundled (`npm run bundle`).
3.  **Verification**: You MUST execute `just verify` and achieve SUCCESS. This consolidated gate enforces:
    - **Formatting & Linting**: `fmt` and `clippy`.
    - **ARCH-GATE**: Physical enforcement of `keyforge_law.yaml` via `sg scan` and `bouncer_100x.py`.
    - **Hexagonal Purity**: Enforcement of layer boundaries via `check_arch.py`.
    - **Logic Truth**: Execution of `test-all`.
4.  **Knowledge Persistence**: Use `store_knowledge` to record a 5-line semantic summary of your work in the Agent Registry memory.
5.  **Final Act**: The `STATUS_UPDATE` message MUST be the final tool call of the session. An agent is not "Done" until the report is physically in the mailbox.

## Testing
- **kf_test**: All tests MUST use `#[keyforge_testing_macros::kf_test]`. This macro allows Clippy to distinguish test code (where `unwrap()` is permitted) from production code (where it is forbidden). Use of standard `#[test]` is a LAW VIOLATION (TEST-001).

## Workflow
- **DEBUG-LOOP (Stuck Rule)**: If you fail `just verify` twice with the same error, or if a task takes > 3 turns without progress, you MUST:
    1. **STOP** implementation.
    2. **THINK**: Explain the failure and the repeated mistake in your internal session.
    3. **ASK**: Dispatch a `QUERY_REQUEST` to the Conductor for guidance before making a 3rd attempt.
- **Test-first (SAGA loop)**: Isolate debugging in `repro.rs`, revert if fix fails twice.
